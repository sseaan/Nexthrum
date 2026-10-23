//! Bridge contract tests.

#[cfg(test)]
mod contract {
    use crate::dto::{BridgeError, BridgeMessage, BridgeSettings, ConnectionState};
    use crate::events::{map_gotify_event, with_unread_snapshot};
    use crate::service::BridgeService;
    use crate::settings::validate_settings;
    use chrono::Utc;
    use nexthrum_core::message_service::MessageRepository;
    use nexthrum_core::{AppSettings, GotifyMessage};
    use nexthrum_storage::StoragePaths;

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
    use crate::dto::{BridgeApplication, BridgeError, BridgeSettings};
    use crate::service::BridgeService;

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
            Err(BridgeError::MissingServerUrl)
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
            BridgeError::NetworkUnavailable
                | BridgeError::RequestTimeout
                | BridgeError::ServerRejected
        ));
    }
}
