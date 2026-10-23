//! Message queries and mutations exposed to the native clients.

use chrono::{DateTime, Utc};
use nexthrum_core::{AppSettings, GotifyMessage};

use crate::dto::{BridgeError, BridgeMessage, BridgeMessageSnapshot, PreparedMessage};
use crate::service::BridgeService;
use crate::settings::configured_storage_paths;

#[uniffi::export]
impl BridgeService {
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
