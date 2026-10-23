//! Streaming connection lifecycle and diagnostics.

use chrono::Utc;
use nexthrum_core::message_service::MessageRepository;
use nexthrum_core::AppSettings;
use std::sync::mpsc;
use std::thread;

use crate::dto::{BridgeApplication, BridgeError, BridgeEvent, ConnectionState};
use crate::events::{map_gotify_event, with_unread_snapshot};
use crate::service::BridgeService;

#[uniffi::export]
impl BridgeService {
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
}
