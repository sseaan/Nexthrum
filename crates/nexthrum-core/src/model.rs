use serde::{Deserialize, Serialize};

/// Application settings persisted by the host adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub server_url: String,
    pub client_token: String,
    pub auto_start: bool,
    pub start_hidden: bool,
    pub minimize_to_tray: bool,
    pub history_retention_days: u32,
    pub do_not_disturb_start: String,
    pub do_not_disturb_end: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            server_url: String::new(),
            client_token: String::new(),
            auto_start: false,
            start_hidden: false,
            minimize_to_tray: true,
            history_retention_days: 7,
            do_not_disturb_start: "22:00".to_string(),
            do_not_disturb_end: "08:00".to_string(),
        }
    }
}

/// A Gotify message with local read/receive metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GotifyMessage {
    pub id: i64,
    pub appid: i64,
    pub message: String,
    pub title: String,
    pub priority: i64,
    pub date: String,
    #[serde(default)]
    pub extras: Option<serde_json::Value>,
    #[serde(default)]
    pub read: bool,
    #[serde(default)]
    pub received_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageSnapshot {
    pub messages: Vec<MessageListItem>,
    pub unread_count: i64,
}

/// A Gotify application (message source). Only the fields the UI needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GotifyApplication {
    pub id: i64,
    pub name: String,
}

/// Message DTO displayed by desktop clients.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageListItem {
    pub id: i64,
    pub appid: i64,
    pub message: String,
    pub title: String,
    pub priority: i64,
    pub date: String,
    pub read: bool,
    pub received_at: String,
}

#[cfg(test)]
mod tests {
    use super::{AppSettings, GotifyMessage, MessageListItem, MessageSnapshot};

    /// Field names are a wire contract shared with TypeScript (`src/types`) and
    /// Swift (`BridgeMessage`). Renaming a field must update both hosts.
    fn keys(value: &serde_json::Value) -> Vec<String> {
        let mut keys: Vec<String> = value
            .as_object()
            .expect("payload must be a JSON object")
            .keys()
            .cloned()
            .collect();
        keys.sort();
        keys
    }

    fn message() -> GotifyMessage {
        GotifyMessage {
            id: 1,
            appid: 2,
            message: "body".into(),
            title: "title".into(),
            priority: 5,
            date: "2026-01-01T00:00:00Z".into(),
            extras: None,
            read: false,
            received_at: "2026-01-01T00:00:01Z".into(),
        }
    }

    #[test]
    fn gotify_message_event_payload_keeps_its_wire_shape() {
        let serialized = serde_json::to_value(message()).unwrap();
        assert_eq!(
            keys(&serialized),
            vec![
                "appid",
                "date",
                "extras",
                "id",
                "message",
                "priority",
                "read",
                "received_at",
                "title"
            ]
        );
        // Deserialization must tolerate payloads without local metadata.
        let incoming: GotifyMessage = serde_json::from_str(
            r#"{"id":7,"appid":2,"message":"m","title":"t","priority":3,"date":"2026-01-01T00:00:00Z"}"#,
        )
        .unwrap();
        assert!(!incoming.read);
        assert!(incoming.extras.is_none());
        assert!(incoming.received_at.is_empty());
    }

    #[test]
    fn message_list_item_and_snapshot_keep_their_wire_shape() {
        let item = MessageListItem {
            id: 1,
            appid: 2,
            message: "body".into(),
            title: "title".into(),
            priority: 5,
            date: "2026-01-01T00:00:00Z".into(),
            read: true,
            received_at: "2026-01-01T00:00:01Z".into(),
        };
        assert_eq!(
            keys(&serde_json::to_value(&item).unwrap()),
            vec![
                "appid",
                "date",
                "id",
                "message",
                "priority",
                "read",
                "received_at",
                "title"
            ]
        );

        let snapshot = MessageSnapshot {
            messages: vec![item],
            unread_count: 0,
        };
        assert_eq!(
            keys(&serde_json::to_value(&snapshot).unwrap()),
            vec!["messages", "unread_count"]
        );
    }

    #[test]
    fn settings_wire_shape_matches_the_frontend_interface() {
        let serialized = serde_json::to_value(AppSettings::default()).unwrap();
        assert_eq!(
            keys(&serialized),
            vec![
                "auto_start",
                "client_token",
                "do_not_disturb_end",
                "do_not_disturb_start",
                "history_retention_days",
                "minimize_to_tray",
                "server_url",
                "start_hidden"
            ]
        );
        // Unknown fields from newer builds must not break older installs.
        let parsed: AppSettings =
            serde_json::from_str(r#"{"server_url":"https://example.test","future_field":1}"#)
                .unwrap();
        assert_eq!(parsed.server_url, "https://example.test");
        assert_eq!(parsed.history_retention_days, 7);
    }
}
