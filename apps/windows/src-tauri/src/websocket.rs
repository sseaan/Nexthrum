use crate::{AppState, GotifyMessage};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{mpsc, watch};

/// Handle to control a running host-side Gotify event forwarder.
pub struct WsHandle {
    pub task: tokio::task::JoinHandle<()>,
    pub shutdown: watch::Sender<bool>,
}

/// Start the platform-independent Gotify client and adapt its events to Tauri.
pub async fn connect(
    app_handle: &AppHandle,
    server_url: &str,
    client_token: &str,
) -> Result<(), String> {
    let (event_tx, mut event_rx) = mpsc::channel(64);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let url = server_url.to_string();
    let token = client_token.to_string();
    let app_handle = app_handle.clone();
    let task_app_handle = app_handle.clone();

    let task = tokio::spawn(async move {
        let client = crate::gotify::run_websocket_client(url, token, event_tx, shutdown_rx);
        tokio::pin!(client);

        loop {
            tokio::select! {
                event = event_rx.recv() => {
                    match event {
                        Some(crate::gotify::GotifyEvent::Connecting) => {
                            let _ = task_app_handle.emit("connection-state-changed", "connecting");
                        }
                        Some(crate::gotify::GotifyEvent::Connected) => {
                            let _ = task_app_handle.emit("connection-state-changed", "connected");
                        }
                        Some(crate::gotify::GotifyEvent::Disconnected { reason }) => {
                            if let Some(reason) = &reason {
                                log::warn!("Gotify stream disconnected: {reason}");
                                let _ = task_app_handle.emit("connection-error", reason);
                            }
                            let _ = task_app_handle.emit("connection-state-changed", "disconnected");
                        }
                        Some(crate::gotify::GotifyEvent::Message(message)) => {
                            handle_incoming_message(&task_app_handle, message).await;
                        }
                        None => break,
                    }
                }
                _ = &mut client => {
                    while let Ok(event) = event_rx.try_recv() {
                        if let crate::gotify::GotifyEvent::Disconnected { reason } = event {
                            if let Some(reason) = reason {
                                log::warn!("Gotify stream disconnected: {reason}");
                                let _ = task_app_handle.emit("connection-error", reason);
                            }
                            let _ = task_app_handle.emit("connection-state-changed", "disconnected");
                        }
                    }
                    break;
                }
            }
        }
    });

    let state = app_handle.state::<AppState>();
    let mut current = state.ws_handle.lock().await;
    if let Some(old) = current.take() {
        let _ = old.shutdown.send(true);
        old.task.abort();
        let _ = old.task.await;
    }
    *current = Some(WsHandle {
        task,
        shutdown: shutdown_tx,
    });
    Ok(())
}

pub async fn reconnect(
    app_handle: &AppHandle,
    server_url: &str,
    client_token: &str,
) -> Result<(), String> {
    disconnect(app_handle).await;
    connect(app_handle, server_url, client_token).await
}

pub async fn disconnect(app_handle: &AppHandle) {
    let state = app_handle.state::<AppState>();
    let mut ws = state.ws_handle.lock().await;
    if let Some(handle) = ws.take() {
        let _ = handle.shutdown.send(true);
        handle.task.abort();
        let _ = handle.task.await;
        log::info!("Gotify WebSocket disconnected");
    }
    let _ = app_handle.emit("connection-state-changed", "disconnected");
}

async fn handle_incoming_message(app_handle: &AppHandle, msg: GotifyMessage) {
    let state = app_handle.state::<AppState>();
    let settings = state.settings.lock().await.clone();
    let paths = state.storage_paths.clone();
    let mut repository = nexthrum_storage::SqliteMessageRepository { paths };
    let result = nexthrum_core::message_service::receive_message(
        &mut repository,
        &settings,
        msg,
        chrono::Utc::now(),
        nexthrum_core::message_policy::local_minute_of_day(chrono::Local::now()),
    );
    let result = match result {
        Ok(result) => result,
        Err(error) => {
            log::error!("Failed to process incoming Gotify message: {error}");
            return;
        }
    };

    let _ = app_handle.emit("new-message", &result.message);
    crate::emit_unread(app_handle, result.unread_count);
    if !result.should_notify {
        log::debug!(
            "In DND period, saved message {} without notification",
            result.message.id
        );
        return;
    }

    let app_handle = app_handle.clone();
    let title = result.message.title;
    let message = result.message.message;
    tauri::async_runtime::spawn(async move {
        if let Err(error) = tauri_plugin_notification::NotificationExt::notification(&app_handle)
            .builder()
            .title(&title)
            .body(&message)
            .show()
        {
            log::warn!("Failed to show notification: {error:?}");
        }
    });
}
