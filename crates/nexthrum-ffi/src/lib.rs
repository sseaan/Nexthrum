use chrono::{DateTime, Utc};
use nexthrum_core::message_service::MessageRepository;
use nexthrum_core::{AppSettings, GotifyMessage};
use nexthrum_storage::StoragePaths;
use serde::{Deserialize, Serialize};
use std::sync::{mpsc, Mutex};
use std::thread;

/// Stable connection state intended for Swift/UniFFI mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum, Default)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
}

/// Stable settings DTO. It intentionally does not expose storage or platform fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct BridgeSettings {
    pub server_url: String,
    pub client_token: String,
    pub auto_start: bool,
    pub history_retention_days: u32,
    pub do_not_disturb_start: String,
    pub do_not_disturb_end: String,
}

/// Stable message DTO for native clients.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct BridgeMessage {
    pub id: i64,
    pub app_id: i64,
    pub title: String,
    pub body: String,
    pub priority: i64,
    pub date: String,
    pub read: bool,
    pub received_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct BridgeApplication {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct BridgeMessageSnapshot {
    pub messages: Vec<BridgeMessage>,
    pub unread_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct PreparedMessage {
    pub message: BridgeMessage,
    pub should_notify: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, uniffi::Record)]
pub struct BridgeEvent {
    pub state: ConnectionState,
    pub message: Option<BridgeMessage>,
    pub should_notify: bool,
    pub unread_count: i64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Error)]
pub enum BridgeError {
    MissingServerUrl,
    MissingClientToken,
    InvalidServerUrl,
    InvalidDndTime,
    InvalidReceivedAt,
    ConnectionFailed,
    Unauthorized,
    ServerNotFound,
    ServerRejected,
    NetworkUnavailable,
    RequestTimeout,
    TlsFailure,
    InvalidResponse,
    ConnectionAlreadyRunning,
    StorageNotConfigured,
    InvalidStoragePath,
    StorageFailed,
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::MissingServerUrl => "Server URL is required",
            Self::MissingClientToken => "Client token is required",
            Self::InvalidServerUrl => "Server URL must use HTTP or HTTPS",
            Self::InvalidDndTime => "Do-not-disturb time must use HH:MM format",
            Self::InvalidReceivedAt => "Received timestamp must use RFC3339 format",
            Self::ConnectionFailed => "Gotify connection test failed",
            Self::Unauthorized => "The Gotify server rejected the client token",
            Self::ServerNotFound => "The Gotify API endpoint was not found; check the server URL",
            Self::ServerRejected => "The Gotify server rejected the request",
            Self::NetworkUnavailable => "Cannot reach the Gotify server",
            Self::RequestTimeout => "The Gotify server did not respond in time",
            Self::TlsFailure => "TLS handshake with the Gotify server failed",
            Self::InvalidResponse => "The Gotify server returned an unexpected response",
            Self::ConnectionAlreadyRunning => "Gotify connection is already running",
            Self::StorageNotConfigured => "Storage paths must be configured before connecting",
            Self::InvalidStoragePath => "Storage paths are invalid",
            Self::StorageFailed => "Message storage operation failed",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for BridgeError {}

impl From<nexthrum_gotify::GotifyError> for BridgeError {
    fn from(error: nexthrum_gotify::GotifyError) -> Self {
        use nexthrum_gotify::GotifyError;
        match error {
            GotifyError::InvalidUrl => BridgeError::InvalidServerUrl,
            GotifyError::Unauthorized => BridgeError::Unauthorized,
            GotifyError::NotFound => BridgeError::ServerNotFound,
            GotifyError::HttpStatus(_) => BridgeError::ServerRejected,
            GotifyError::InvalidResponse(_) => BridgeError::InvalidResponse,
            GotifyError::Network(_) => BridgeError::NetworkUnavailable,
            GotifyError::Timeout => BridgeError::RequestTimeout,
            GotifyError::Tls(_) => BridgeError::TlsFailure,
        }
    }
}

impl From<&BridgeSettings> for AppSettings {
    fn from(settings: &BridgeSettings) -> Self {
        Self {
            server_url: settings.server_url.clone(),
            client_token: settings.client_token.clone(),
            auto_start: settings.auto_start,
            history_retention_days: settings.history_retention_days,
            do_not_disturb_start: settings.do_not_disturb_start.clone(),
            do_not_disturb_end: settings.do_not_disturb_end.clone(),
            ..Self::default()
        }
    }
}

impl From<&GotifyMessage> for BridgeMessage {
    fn from(message: &GotifyMessage) -> Self {
        Self {
            id: message.id,
            app_id: message.appid,
            title: message.title.clone(),
            body: message.message.clone(),
            priority: message.priority,
            date: message.date.clone(),
            read: message.read,
            received_at: message.received_at.clone(),
        }
    }
}

fn map_gotify_event<R: nexthrum_core::message_service::MessageRepository>(
    event: nexthrum_gotify::GotifyEvent,
    settings: &AppSettings,
    repository: &mut R,
    received_at: DateTime<Utc>,
    local_minute_of_day: u32,
) -> BridgeEvent {
    match event {
        nexthrum_gotify::GotifyEvent::Connecting => BridgeEvent {
            state: ConnectionState::Connecting,
            message: None,
            should_notify: false,
            unread_count: 0,
            error: None,
        },
        nexthrum_gotify::GotifyEvent::Connected => BridgeEvent {
            state: ConnectionState::Connected,
            message: None,
            should_notify: false,
            unread_count: 0,
            error: None,
        },
        nexthrum_gotify::GotifyEvent::Disconnected { reason } => BridgeEvent {
            state: ConnectionState::Disconnected,
            message: None,
            should_notify: false,
            unread_count: 0,
            error: reason,
        },
        nexthrum_gotify::GotifyEvent::Message(message) => {
            match nexthrum_core::message_service::receive_message(
                repository,
                settings,
                message,
                received_at,
                local_minute_of_day,
            ) {
                Ok(result) => BridgeEvent {
                    state: ConnectionState::Connected,
                    message: Some(BridgeMessage::from(&result.message)),
                    should_notify: result.should_notify,
                    unread_count: result.unread_count,
                    error: None,
                },
                Err(error) => BridgeEvent {
                    state: ConnectionState::Connected,
                    message: None,
                    should_notify: false,
                    unread_count: repository.unread_count().unwrap_or(0),
                    error: Some(error),
                },
            }
        }
    }
}

fn with_unread_snapshot(mut event: BridgeEvent, unread_count: i64) -> BridgeEvent {
    event.unread_count = unread_count;
    event
}

/// Native bridge service. The Gotify network lifecycle runs on a private Rust thread;
/// events cross the FFI boundary only as stable DTO values.
#[derive(Debug, uniffi::Object)]
pub struct BridgeService {
    settings: Mutex<Option<BridgeSettings>>,
    storage_paths: Mutex<Option<StoragePaths>>,
    state: Mutex<ConnectionState>,
    event_receiver: Mutex<Option<mpsc::Receiver<BridgeEvent>>>,
    shutdown_sender: Mutex<Option<tokio::sync::watch::Sender<bool>>>,
    worker_handle: Mutex<Option<thread::JoinHandle<()>>>,
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

    pub fn get_messages(&self) -> Result<Vec<BridgeMessage>, BridgeError> {
        let paths = self
            .storage_paths
            .lock()
            .unwrap()
            .clone()
            .ok_or(BridgeError::StorageNotConfigured)?;
        let repository = nexthrum_storage::SqliteMessageRepository { paths };
        let messages = nexthrum_core::message_service::get_messages(&repository)
            .map_err(|_| BridgeError::StorageFailed)?;
        Ok(messages
            .into_iter()
            .map(|message| BridgeMessage {
                id: message.id,
                app_id: message.appid,
                title: message.title,
                body: message.message,
                priority: message.priority,
                date: message.date,
                read: message.read,
                received_at: message.received_at,
            })
            .collect())
    }

    pub fn get_message_snapshot(&self) -> Result<BridgeMessageSnapshot, BridgeError> {
        let paths = configured_storage_paths(&self.storage_paths)?;
        let repository = nexthrum_storage::SqliteMessageRepository { paths };
        let snapshot = nexthrum_core::message_service::get_message_snapshot(&repository)
            .map_err(|_| BridgeError::StorageFailed)?;
        Ok(BridgeMessageSnapshot {
            messages: snapshot
                .messages
                .into_iter()
                .map(|message| BridgeMessage {
                    id: message.id,
                    app_id: message.appid,
                    title: message.title,
                    body: message.message,
                    priority: message.priority,
                    date: message.date,
                    read: message.read,
                    received_at: message.received_at,
                })
                .collect(),
            unread_count: snapshot.unread_count,
        })
    }

    pub fn unread_count(&self) -> Result<i64, BridgeError> {
        let paths = configured_storage_paths(&self.storage_paths)?;
        let repository = nexthrum_storage::SqliteMessageRepository { paths };
        nexthrum_core::message_service::get_unread_count(&repository)
            .map_err(|_| BridgeError::StorageFailed)
    }

    pub fn mark_message_read(&self, message_id: i64) -> Result<i64, BridgeError> {
        let paths = configured_storage_paths(&self.storage_paths)?;
        let mut repository = nexthrum_storage::SqliteMessageRepository { paths };
        nexthrum_core::message_service::mark_message_read(&mut repository, message_id)
            .map_err(|_| BridgeError::StorageFailed)
    }

    pub fn delete_message(&self, message_id: i64) -> Result<i64, BridgeError> {
        let paths = configured_storage_paths(&self.storage_paths)?;
        let mut repository = nexthrum_storage::SqliteMessageRepository { paths };
        nexthrum_core::message_service::delete_message(&mut repository, message_id)
            .map_err(|_| BridgeError::StorageFailed)
    }

    pub fn clear_messages(&self) -> Result<i64, BridgeError> {
        let paths = configured_storage_paths(&self.storage_paths)?;
        let mut repository = nexthrum_storage::SqliteMessageRepository { paths };
        nexthrum_core::message_service::clear_messages(&mut repository)
            .map_err(|_| BridgeError::StorageFailed)
    }

    pub fn settings(&self) -> Option<BridgeSettings> {
        self.settings.lock().unwrap().clone()
    }

    pub fn connection_state(&self) -> ConnectionState {
        *self.state.lock().unwrap()
    }

    pub fn start_connection(&self) -> Result<(), BridgeError> {
        let settings = self.settings().ok_or(BridgeError::MissingServerUrl)?;
        nexthrum_gotify::websocket_url(&settings.server_url, &settings.client_token)
            .map_err(|_| BridgeError::InvalidServerUrl)?;
        let storage_paths = self
            .storage_paths
            .lock()
            .unwrap()
            .clone()
            .ok_or(BridgeError::StorageNotConfigured)?;
        {
            let mut state = self.state.lock().unwrap();
            if *state != ConnectionState::Disconnected {
                return Err(BridgeError::ConnectionAlreadyRunning);
            }
            *state = ConnectionState::Connecting;
        }
        if let Some(shutdown) = self.shutdown_sender.lock().unwrap().take() {
            let _ = shutdown.send(true);
        }
        let previous_worker = self.worker_handle.lock().unwrap().take();
        if let Some(worker) = previous_worker {
            let _ = worker.join();
        }

        let policy_settings = AppSettings::from(&settings);
        let (gotify_tx, mut gotify_rx) = tokio::sync::mpsc::channel(32);
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let (event_tx, event_rx) = mpsc::channel();
        let _ = event_tx.send(BridgeEvent {
            state: ConnectionState::Connecting,
            message: None,
            should_notify: false,
            unread_count: nexthrum_storage::get_unread_count(&storage_paths).unwrap_or(0),
            error: None,
        });
        *self.event_receiver.lock().unwrap() = Some(event_rx);
        *self.shutdown_sender.lock().unwrap() = Some(shutdown_tx);

        let panic_tx = event_tx.clone();
        let worker = thread::spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(_) => {
                    let _ = event_tx.send(BridgeEvent {
                        state: ConnectionState::Disconnected,
                        message: None,
                        should_notify: false,
                        unread_count: nexthrum_storage::get_unread_count(&storage_paths)
                            .unwrap_or(0),
                        error: Some("Failed to start networking runtime".into()),
                    });
                    return;
                }
            };
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                runtime.block_on(async move {
                let client = nexthrum_gotify::run_websocket_client(
                    settings.server_url,
                    settings.client_token,
                    gotify_tx,
                    shutdown_rx,
                );
                tokio::pin!(client);
                let mut repository = nexthrum_storage::SqliteMessageRepository {
                    paths: storage_paths.clone(),
                };
                loop {
                    tokio::select! {
                        event = gotify_rx.recv() => {
                            let Some(event) = event else { break };
                            let mut bridge_event = map_gotify_event(
                                event,
                                &policy_settings,
                                &mut repository,
                                Utc::now(),
                                nexthrum_core::message_policy::local_minute_of_day(chrono::Local::now()),
                            );
                            if bridge_event.message.is_none() {
                                bridge_event = with_unread_snapshot(
                                    bridge_event,
                                    repository.unread_count().unwrap_or(0),
                                );
                            }
                            if event_tx.send(bridge_event).is_err() { break; }
                        }
                        _ = &mut client => {
                            while let Ok(event) = gotify_rx.try_recv() {
                                let bridge_event = with_unread_snapshot(
                                    map_gotify_event(
                                        event,
                                        &policy_settings,
                                        &mut repository,
                                        Utc::now(),
                                        nexthrum_core::message_policy::local_minute_of_day(chrono::Local::now()),
                                    ),
                                    repository.unread_count().unwrap_or(0),
                                );
                                if event_tx.send(bridge_event).is_err() {
                                    break;
                                }
                            }
                            break;
                        }
                    }
                }
                });
            }));

            if let Err(panic) = outcome {
                let detail = panic
                    .downcast_ref::<&str>()
                    .map(|message| (*message).to_string())
                    .or_else(|| panic.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "unknown panic".to_string());
                log::error!("Gotify bridge worker panicked: {detail}");
                let _ = panic_tx.send(BridgeEvent {
                    state: ConnectionState::Disconnected,
                    message: None,
                    should_notify: false,
                    unread_count: 0,
                    error: Some(format!("The Gotify client stopped unexpectedly: {detail}")),
                });
            }
        });
        *self.worker_handle.lock().unwrap() = Some(worker);
        Ok(())
    }

    /// Return the next buffered event, if one is available immediately.
    pub fn poll_event(&self) -> Option<BridgeEvent> {
        let event = self
            .event_receiver
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|receiver| receiver.try_recv().ok());
        if let Some(event) = &event {
            *self.state.lock().unwrap() = event.state;
        }
        event
    }

    /// Stop the client and discard buffered events.
    pub fn stop_connection(&self) {
        if let Some(shutdown) = self.shutdown_sender.lock().unwrap().take() {
            let _ = shutdown.send(true);
        }
        self.event_receiver.lock().unwrap().take();
        *self.state.lock().unwrap() = ConnectionState::Disconnected;
        let worker = self.worker_handle.lock().unwrap().take();
        if let Some(worker) = worker {
            if worker.join().is_err() {
                log::error!("Gotify bridge worker panicked while stopping");
            }
        }
    }

    pub fn test_connection(&self) -> Result<String, BridgeError> {
        let settings = self.settings().ok_or(BridgeError::MissingServerUrl)?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| BridgeError::ConnectionFailed)?;
        runtime
            .block_on(nexthrum_gotify::test_connection(
                &settings.server_url,
                &settings.client_token,
            ))
            .map_err(BridgeError::from)
    }

    /// Resolve Gotify application names for the source filter.
    ///
    /// Streamed messages only carry `appid`; hosts fall back to `App <id>` when
    /// this fails (for example when the server restricts `/application`).
    pub fn list_applications(&self) -> Result<Vec<BridgeApplication>, BridgeError> {
        let settings = self.settings().ok_or(BridgeError::MissingServerUrl)?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| BridgeError::ConnectionFailed)?;
        let applications = runtime
            .block_on(nexthrum_gotify::list_applications(
                &settings.server_url,
                &settings.client_token,
            ))
            .map_err(BridgeError::from)?;
        Ok(applications
            .into_iter()
            .map(|application| BridgeApplication {
                id: application.id,
                name: application.name,
            })
            .collect())
    }

    pub fn should_notify(&self, local_minute_of_day: u32) -> Result<bool, BridgeError> {
        let settings = self.settings().ok_or(BridgeError::MissingServerUrl)?;
        let core_settings = AppSettings::from(&settings);
        Ok(!nexthrum_core::message_policy::is_in_dnd_period(
            &core_settings,
            local_minute_of_day,
        ))
    }

    pub fn prepare_message(
        &self,
        message: BridgeMessage,
        received_at: String,
        local_minute_of_day: u32,
    ) -> Result<PreparedMessage, BridgeError> {
        let settings = self.settings().ok_or(BridgeError::MissingServerUrl)?;
        let core_settings = AppSettings::from(&settings);
        let received_at = DateTime::parse_from_rfc3339(&received_at)
            .map_err(|_| BridgeError::InvalidReceivedAt)?
            .with_timezone(&Utc);
        let core_message = GotifyMessage {
            id: message.id,
            appid: message.app_id,
            message: message.body,
            title: message.title,
            priority: message.priority,
            date: message.date,
            extras: None,
            read: message.read,
            received_at: message.received_at,
        };
        let prepared = nexthrum_core::message_policy::prepare_incoming_message(
            core_message,
            &core_settings,
            received_at,
            local_minute_of_day,
        );
        Ok(PreparedMessage {
            message: BridgeMessage::from(&prepared.message),
            should_notify: prepared.should_notify,
        })
    }
}

impl Drop for BridgeService {
    fn drop(&mut self) {
        self.stop_connection();
    }
}

fn configured_storage_paths(
    paths: &Mutex<Option<StoragePaths>>,
) -> Result<StoragePaths, BridgeError> {
    paths
        .lock()
        .unwrap()
        .clone()
        .ok_or(BridgeError::StorageNotConfigured)
}

pub fn validate_settings(settings: &BridgeSettings) -> Result<(), BridgeError> {
    if settings.server_url.trim().is_empty() {
        return Err(BridgeError::MissingServerUrl);
    }
    if settings.client_token.trim().is_empty() {
        return Err(BridgeError::MissingClientToken);
    }
    if !valid_time(&settings.do_not_disturb_start) || !valid_time(&settings.do_not_disturb_end) {
        return Err(BridgeError::InvalidDndTime);
    }
    Ok(())
}

fn valid_time(time: &str) -> bool {
    let mut parts = time.split(':');
    let (Some(hours), Some(minutes), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    match (hours.parse::<u32>(), minutes.parse::<u32>()) {
        (Ok(hours), Ok(minutes)) => hours < 24 && minutes < 60,
        _ => false,
    }
}

uniffi::setup_scaffolding!();

#[cfg(test)]
mod tests {
    use super::{
        map_gotify_event, validate_settings, with_unread_snapshot, BridgeError, BridgeMessage,
        BridgeService, BridgeSettings, ConnectionState, StoragePaths,
    };
    use chrono::Utc;
    use nexthrum_core::message_service::MessageRepository;
    use nexthrum_core::{AppSettings, GotifyMessage};

    fn settings() -> BridgeSettings {
        BridgeSettings {
            server_url: "https://example.test".into(),
            client_token: "secret".into(),
            auto_start: false,
            history_retention_days: 7,
            do_not_disturb_start: "22:00".into(),
            do_not_disturb_end: "08:00".into(),
        }
    }

    #[test]
    fn service_validates_settings_without_exposing_storage() {
        let service = BridgeService::new();
        service.set_settings(settings()).unwrap();
        assert_eq!(service.connection_state(), ConnectionState::Disconnected);
        assert!(!service.should_notify(23 * 60).unwrap());
        assert!(service.should_notify(12 * 60).unwrap());
    }

    #[test]
    fn connection_state_events_keep_authoritative_unread_snapshot() {
        let root = tempfile::tempdir().unwrap();
        let paths = StoragePaths::new(
            root.path().join("messages.sqlite"),
            root.path().join("settings.json"),
        );
        let mut repository = nexthrum_storage::SqliteMessageRepository { paths };
        let settings = AppSettings::default();
        let event = map_gotify_event(
            nexthrum_gotify::GotifyEvent::Disconnected { reason: None },
            &settings,
            &mut repository,
            Utc::now(),
            0,
        );
        let event = with_unread_snapshot(event, 7);
        assert_eq!(event.state, ConnectionState::Disconnected);
        assert_eq!(event.unread_count, 7);
        assert!(event.message.is_none());
        assert!(event.error.is_none());
    }

    #[test]
    fn disconnect_reason_reaches_the_bridge_event() {
        let root = tempfile::tempdir().unwrap();
        let paths = StoragePaths::new(
            root.path().join("messages.sqlite"),
            root.path().join("settings.json"),
        );
        let mut repository = nexthrum_storage::SqliteMessageRepository { paths };
        let event = map_gotify_event(
            nexthrum_gotify::GotifyEvent::Disconnected {
                reason: Some("TLS handshake with the Gotify server failed".into()),
            },
            &AppSettings::default(),
            &mut repository,
            Utc::now(),
            0,
        );
        assert_eq!(event.state, ConnectionState::Disconnected);
        assert_eq!(
            event.error.as_deref(),
            Some("TLS handshake with the Gotify server failed")
        );
    }

    #[test]
    fn gotify_errors_map_to_actionable_bridge_errors() {
        use nexthrum_gotify::GotifyError;
        assert_eq!(
            BridgeError::from(GotifyError::InvalidUrl),
            BridgeError::InvalidServerUrl
        );
        assert_eq!(
            BridgeError::from(GotifyError::Unauthorized),
            BridgeError::Unauthorized
        );
        assert_eq!(
            BridgeError::from(GotifyError::NotFound),
            BridgeError::ServerNotFound
        );
        assert_eq!(
            BridgeError::from(GotifyError::HttpStatus(502)),
            BridgeError::ServerRejected
        );
        assert_eq!(
            BridgeError::from(GotifyError::Network("refused".into())),
            BridgeError::NetworkUnavailable
        );
        assert_eq!(
            BridgeError::from(GotifyError::Timeout),
            BridgeError::RequestTimeout
        );
        assert_eq!(
            BridgeError::from(GotifyError::Tls("bad cert".into())),
            BridgeError::TlsFailure
        );
        assert_eq!(
            BridgeError::from(GotifyError::InvalidResponse("html".into())),
            BridgeError::InvalidResponse
        );
        // Messages must stay distinct so the UI can guide the user.
        assert_ne!(
            BridgeError::Unauthorized.to_string(),
            BridgeError::NetworkUnavailable.to_string()
        );
    }

    #[test]
    fn invalid_settings_have_stable_errors() {
        let mut invalid = settings();
        invalid.client_token.clear();
        assert_eq!(
            validate_settings(&invalid),
            Err(BridgeError::MissingClientToken)
        );
        invalid.client_token = "token".into();
        invalid.do_not_disturb_start = "25:00".into();
        assert_eq!(
            validate_settings(&invalid),
            Err(BridgeError::InvalidDndTime)
        );
    }

    #[test]
    fn incoming_event_applies_dnd_and_local_read_policy() {
        let settings = AppSettings::from(&settings());
        let message = GotifyMessage {
            id: 7,
            appid: 1,
            message: "body".into(),
            title: "title".into(),
            priority: 1,
            date: "2026-01-01T00:00:00Z".into(),
            extras: None,
            read: true,
            received_at: String::new(),
        };
        let root = tempfile::tempdir().unwrap();
        let paths = StoragePaths::new(
            root.path().join("messages.sqlite"),
            root.path().join("settings.json"),
        );
        let mut repository = nexthrum_storage::SqliteMessageRepository { paths };
        let event = map_gotify_event(
            nexthrum_gotify::GotifyEvent::Message(message),
            &settings,
            &mut repository,
            Utc::now(),
            23 * 60,
        );
        let persisted = event.message.unwrap();
        assert!(!persisted.read);
        assert_eq!(repository.unread_count().unwrap(), 1);
    }

    #[test]
    fn bridge_reads_messages_from_injected_storage() {
        let service = BridgeService::new();
        let root = tempfile::tempdir().unwrap();
        let database_path = root.path().join("db/messages.sqlite");
        let settings_path = root.path().join("prefs/settings.json");
        service
            .set_storage_paths(
                database_path.display().to_string(),
                settings_path.display().to_string(),
            )
            .unwrap();
        let paths = StoragePaths::new(database_path, settings_path);
        let message = GotifyMessage {
            id: 18,
            appid: 4,
            message: "persisted".into(),
            title: "saved".into(),
            priority: 2,
            date: "2026-10-22T12:00:00Z".into(),
            extras: None,
            read: false,
            received_at: "2026-10-22T12:00:00Z".into(),
        };
        nexthrum_storage::save_message(&paths, &message).unwrap();
        let messages = service.get_messages().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, 18);
        assert_eq!(messages[0].body, "persisted");
        let snapshot = service.get_message_snapshot().unwrap();
        assert_eq!(snapshot.messages.len(), 1);
        assert_eq!(snapshot.unread_count, 1);
        assert_eq!(service.unread_count().unwrap(), 1);
    }

    #[test]
    fn loading_settings_scrubs_legacy_plaintext_token() {
        let service = BridgeService::new();
        let root = tempfile::tempdir().unwrap();
        let settings_path = root.path().join("settings.json");
        std::fs::write(
            &settings_path,
            r#"{"server_url":"https://example.test","client_token":"old-secret"}"#,
        )
        .unwrap();
        service
            .set_storage_paths(
                root.path().join("messages.db").display().to_string(),
                settings_path.display().to_string(),
            )
            .unwrap();
        let loaded = service.load_settings().unwrap().unwrap();
        assert!(loaded.client_token.is_empty());
        let persisted = std::fs::read_to_string(settings_path).unwrap();
        assert!(!persisted.contains("old-secret"));
    }

    #[test]
    fn bridge_persists_settings_without_client_token() {
        let service = BridgeService::new();
        let root = tempfile::tempdir().unwrap();
        let settings_path = root.path().join("settings.json");
        service
            .set_storage_paths(
                root.path().join("messages.db").display().to_string(),
                settings_path.display().to_string(),
            )
            .unwrap();
        let mut configured = settings();
        configured.client_token = "do-not-write-this-secret".into();
        service.set_settings(configured).unwrap();
        let contents = std::fs::read_to_string(settings_path).unwrap();
        assert!(!contents.contains("do-not-write-this-secret"));
        assert!(contents.contains("\"client_token\": \"\""));
    }

    #[test]
    fn connection_lifecycle_is_single_start_and_cancellable() {
        let service = BridgeService::new();
        let mut configured = settings();
        configured.server_url = "http://127.0.0.1:1".into();
        service.set_settings(configured).unwrap();
        service
            .set_storage_paths(
                std::env::temp_dir()
                    .join(format!("nexthrum-ffi-{}-messages.db", std::process::id()))
                    .display()
                    .to_string(),
                std::env::temp_dir()
                    .join(format!("nexthrum-ffi-{}-settings.json", std::process::id()))
                    .display()
                    .to_string(),
            )
            .unwrap();

        service.start_connection().unwrap();
        assert_eq!(service.connection_state(), ConnectionState::Connecting);
        assert_eq!(
            service.start_connection().unwrap_err(),
            BridgeError::ConnectionAlreadyRunning
        );
        service.stop_connection();
        assert_eq!(service.connection_state(), ConnectionState::Disconnected);
        assert!(service.worker_handle.lock().unwrap().is_none());
        service.start_connection().unwrap();
        service.stop_connection();
        assert!(service.worker_handle.lock().unwrap().is_none());
    }

    #[test]
    fn connection_rejects_invalid_server_url_before_starting() {
        let service = BridgeService::new();
        let mut configured = settings();
        configured.server_url = "file:///tmp/not-gotify".into();
        service.set_settings(configured).unwrap();
        assert_eq!(
            service.start_connection().unwrap_err(),
            BridgeError::InvalidServerUrl
        );
        assert_eq!(service.connection_state(), ConnectionState::Disconnected);
    }

    #[test]
    fn prepared_message_maps_to_native_dto() {
        let service = BridgeService::new();
        service.set_settings(settings()).unwrap();
        let message = BridgeMessage {
            id: 1,
            app_id: 2,
            title: "title".into(),
            body: "body".into(),
            priority: 5,
            date: "2026-01-01".into(),
            read: true,
            received_at: String::new(),
        };
        let result = service
            .prepare_message(message, "2026-01-01T12:00:00Z".into(), 12 * 60)
            .unwrap();
        let dto = result.message;
        assert_eq!(dto.app_id, 2);
        assert_eq!(dto.body, "body");
        assert!(!dto.read);
        assert!(result.should_notify);
    }
}

#[cfg(test)]
mod application_tests {
    use super::{BridgeApplication, BridgeService, BridgeSettings};

    fn settings(server_url: &str) -> BridgeSettings {
        BridgeSettings {
            server_url: server_url.into(),
            client_token: "token".into(),
            auto_start: false,
            history_retention_days: 7,
            do_not_disturb_start: "22:00".into(),
            do_not_disturb_end: "08:00".into(),
        }
    }

    #[test]
    fn application_dto_keeps_the_wire_shape() {
        let application = BridgeApplication {
            id: 2,
            name: "CI".into(),
        };
        assert_eq!(application.id, 2);
        assert_eq!(application.name, "CI");
    }

    #[test]
    fn listing_applications_without_settings_is_a_stable_error() {
        let service = BridgeService::new();
        assert_eq!(
            service.list_applications(),
            Err(super::BridgeError::MissingServerUrl)
        );
    }

    #[test]
    fn listing_applications_reports_transport_failures() {
        let service = BridgeService::new();
        // Port 1 refuses immediately.
        service
            .set_settings(settings("http://127.0.0.1:1"))
            .unwrap();
        let error = service.list_applications().unwrap_err();
        // A local proxy may answer for the closed port, so a rejected status is
        // also acceptable; a silent success is not.
        assert!(matches!(
            error,
            super::BridgeError::NetworkUnavailable
                | super::BridgeError::RequestTimeout
                | super::BridgeError::ServerRejected
        ));
    }
}
