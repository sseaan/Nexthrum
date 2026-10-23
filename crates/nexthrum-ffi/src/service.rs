//! Bridge object: construction, lifecycle and settings persistence.

use nexthrum_core::AppSettings;
use nexthrum_storage::StoragePaths;
use std::sync::{mpsc, Mutex};
use std::thread;

use crate::dto::{BridgeError, BridgeEvent, BridgeSettings, ConnectionState};
use crate::settings::{configured_storage_paths, valid_time, validate_settings};

/// Native bridge service. The Gotify network lifecycle runs on a private Rust thread;
/// events cross the FFI boundary only as stable DTO values.
#[derive(Debug, uniffi::Object)]
pub struct BridgeService {
    pub(crate) settings: Mutex<Option<BridgeSettings>>,
    pub(crate) storage_paths: Mutex<Option<StoragePaths>>,
    pub(crate) state: Mutex<ConnectionState>,
    pub(crate) event_receiver: Mutex<Option<mpsc::Receiver<BridgeEvent>>>,
    pub(crate) shutdown_sender: Mutex<Option<tokio::sync::watch::Sender<bool>>>,
    pub(crate) worker_handle: Mutex<Option<thread::JoinHandle<()>>>,
}

impl Default for BridgeService {
    fn default() -> Self {
        Self::new()
    }
}

#[uniffi::export]
impl BridgeService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            settings: Mutex::new(None),
            storage_paths: Mutex::new(None),
            state: Mutex::new(ConnectionState::Disconnected),
            event_receiver: Mutex::new(None),
            shutdown_sender: Mutex::new(None),
            worker_handle: Mutex::new(None),
        }
    }

    pub fn settings(&self) -> Option<BridgeSettings> {
        self.settings.lock().unwrap().clone()
    }

    pub fn set_settings(&self, settings: BridgeSettings) -> Result<(), BridgeError> {
        validate_settings(&settings)?;
        let storage_paths = self.storage_paths.lock().unwrap().clone();
        if let Some(paths) = storage_paths {
            let mut persisted_settings = AppSettings::from(&settings);
            // Native hosts keep credentials in the platform credential vault.
            persisted_settings.client_token.clear();
            nexthrum_storage::save_settings(&paths, &persisted_settings)
                .map_err(|_| BridgeError::StorageFailed)?;
        }
        *self.settings.lock().unwrap() = Some(settings);
        Ok(())
    }

    pub fn load_settings(&self) -> Result<Option<BridgeSettings>, BridgeError> {
        let paths = configured_storage_paths(&self.storage_paths)?;
        let Some(mut settings) = nexthrum_storage::load_settings(&paths) else {
            return Ok(None);
        };
        // Scrub credentials left by earlier bridge builds before returning settings.
        if !settings.client_token.is_empty() {
            settings.client_token.clear();
            nexthrum_storage::save_settings(&paths, &settings)
                .map_err(|_| BridgeError::StorageFailed)?;
        }
        let bridge_settings = BridgeSettings {
            server_url: settings.server_url,
            // Credentials are intentionally never restored from settings JSON.
            client_token: String::new(),
            auto_start: settings.auto_start,
            history_retention_days: settings.history_retention_days,
            do_not_disturb_start: settings.do_not_disturb_start,
            do_not_disturb_end: settings.do_not_disturb_end,
        };
        validate_settings(&bridge_settings).or_else(|error| {
            if bridge_settings.client_token.is_empty()
                && !bridge_settings.server_url.trim().is_empty()
                && valid_time(&bridge_settings.do_not_disturb_start)
                && valid_time(&bridge_settings.do_not_disturb_end)
            {
                Ok(())
            } else {
                Err(error)
            }
        })?;
        *self.settings.lock().unwrap() = Some(bridge_settings.clone());
        Ok(Some(bridge_settings))
    }

    pub fn set_storage_paths(
        &self,
        database_path: String,
        settings_path: String,
    ) -> Result<(), BridgeError> {
        if database_path.trim().is_empty() || settings_path.trim().is_empty() {
            return Err(BridgeError::InvalidStoragePath);
        }
        *self.storage_paths.lock().unwrap() = Some(StoragePaths::new(
            database_path.into(),
            settings_path.into(),
        ));
        Ok(())
    }
}

impl Drop for BridgeService {
    fn drop(&mut self) {
        self.stop_connection();
    }
}
