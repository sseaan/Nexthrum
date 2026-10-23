//! Settings validation and storage-path helpers.

use nexthrum_storage::StoragePaths;
use std::sync::Mutex;

use crate::dto::{BridgeError, BridgeSettings};

pub(crate) fn configured_storage_paths(
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

pub(crate) fn valid_time(time: &str) -> bool {
    let mut parts = time.split(':');
    let (Some(hours), Some(minutes), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    match (hours.parse::<u32>(), minutes.parse::<u32>()) {
        (Ok(hours), Ok(minutes)) => hours < 24 && minutes < 60,
        _ => false,
    }
}
