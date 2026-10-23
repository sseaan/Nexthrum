//! DTOs and error types shared with Swift through UniFFI.

use nexthrum_core::{AppSettings, GotifyMessage};
use serde::{Deserialize, Serialize};

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
