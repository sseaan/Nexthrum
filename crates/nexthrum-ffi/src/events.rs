//! Translate shared-client events into bridge events.

use chrono::{DateTime, Utc};
use nexthrum_core::AppSettings;

use crate::dto::{BridgeEvent, BridgeMessage, ConnectionState};

pub(crate) fn map_gotify_event<R: nexthrum_core::message_service::MessageRepository>(
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

pub(crate) fn with_unread_snapshot(mut event: BridgeEvent, unread_count: i64) -> BridgeEvent {
    event.unread_count = unread_count;
    event
}
