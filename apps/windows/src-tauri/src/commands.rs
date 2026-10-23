use crate::{credentials, tray, websocket, AppSettings, AppState};
use tauri::{Emitter, State};

#[tauri::command]
pub async fn save_settings(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    let _connection_update = state.connection_update.lock().await;
    credentials::save_settings(&state.storage_paths, &settings).map_err(|e| e.to_string())?;
    {
        let mut current = state.settings.lock().await;
        *current = settings.clone();
    }

    // Applied synchronously so a failure can be reported; the setting itself is
    // already persisted and must not be rolled back.
    let autostart_error = tray::set_autostart(&app_handle, settings.auto_start)
        .err()
        .map(|error| {
            log::warn!("Launch-at-login update failed: {error}");
            error
        });

    let server_url = settings.server_url.clone();
    let client_token = settings.client_token.clone();
    if server_url.trim().is_empty() || client_token.trim().is_empty() {
        websocket::disconnect(&app_handle).await;
        if let Some(error) = autostart_error {
            return Err(format!(
                "Settings were saved, but launch at login could not be updated: {error}"
            ));
        }
        return Ok(());
    }
    if let Err(error) = websocket::reconnect(&app_handle, &server_url, &client_token).await {
        let message = format!(
            "Settings were saved, but the Gotify connection could not be restarted: {error}"
        );
        log::error!("{message}");
        let _ = app_handle.emit("connection-error", &message);
        return Err(match autostart_error {
            Some(autostart) => format!("{message}; launch at login also failed: {autostart}"),
            None => message,
        });
    }

    if let Some(error) = autostart_error {
        return Err(format!(
            "Settings were saved, but launch at login could not be updated: {error}"
        ));
    }
    Ok(())
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let settings = state.settings.lock().await;
    Ok(settings.clone())
}

#[tauri::command]
pub async fn get_message_snapshot(
    state: State<'_, AppState>,
) -> Result<nexthrum_core::MessageSnapshot, String> {
    let repository = nexthrum_storage::SqliteMessageRepository {
        paths: state.storage_paths.clone(),
    };
    nexthrum_core::message_service::get_message_snapshot(&repository)
}

#[tauri::command]
pub async fn mark_message_read(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    message_id: i64,
) -> Result<nexthrum_core::MessageSnapshot, String> {
    let _message_update = state.message_update.lock().await;
    let mut repository = nexthrum_storage::SqliteMessageRepository {
        paths: state.storage_paths.clone(),
    };
    let snapshot =
        nexthrum_core::message_service::mark_message_read_snapshot(&mut repository, message_id)?;
    let _ = app_handle.emit("unread-count-changed", snapshot.unread_count);
    Ok(snapshot)
}

#[tauri::command]
pub async fn delete_message(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    message_id: i64,
) -> Result<nexthrum_core::MessageSnapshot, String> {
    let _message_update = state.message_update.lock().await;
    let mut repository = nexthrum_storage::SqliteMessageRepository {
        paths: state.storage_paths.clone(),
    };
    let snapshot =
        nexthrum_core::message_service::delete_message_snapshot(&mut repository, message_id)?;
    let _ = app_handle.emit("unread-count-changed", snapshot.unread_count);
    Ok(snapshot)
}

#[tauri::command]
pub async fn clear_messages(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    sync_server: bool,
) -> Result<nexthrum_core::MessageSnapshot, String> {
    let _message_update = state.message_update.lock().await;
    let settings = state.settings.lock().await;

    if sync_server && !settings.server_url.is_empty() && !settings.client_token.is_empty() {
        crate::gotify::delete_all_messages(&settings.server_url, &settings.client_token)
            .await
            .map_err(|error| format!("Gotify server deletion failed: {error}"))?;
    }

    let mut repository = nexthrum_storage::SqliteMessageRepository {
        paths: state.storage_paths.clone(),
    };
    let snapshot = nexthrum_core::message_service::clear_messages_snapshot(&mut repository)?;
    let _ = app_handle.emit("unread-count-changed", snapshot.unread_count);
    Ok(snapshot)
}

#[tauri::command]
pub async fn connect_websocket(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let _connection_update = state.connection_update.lock().await;
    let settings = state.settings.lock().await;
    if settings.server_url.is_empty() || settings.client_token.is_empty() {
        return Err("Server URL and client token must be configured".to_string());
    }
    let server_url = settings.server_url.clone();
    let client_token = settings.client_token.clone();
    drop(settings);

    websocket::reconnect(&app_handle, &server_url, &client_token)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn disconnect_websocket(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let _connection_update = state.connection_update.lock().await;
    websocket::disconnect(&app_handle).await;
    Ok(())
}

/// Resolve Gotify application names for the source filter.
#[tauri::command]
pub async fn get_applications(
    state: State<'_, AppState>,
) -> Result<Vec<nexthrum_core::GotifyApplication>, String> {
    let settings = state.settings.lock().await;
    if settings.server_url.trim().is_empty() || settings.client_token.trim().is_empty() {
        return Ok(Vec::new());
    }
    let server_url = settings.server_url.clone();
    let client_token = settings.client_token.clone();
    drop(settings);

    crate::gotify::list_applications(&server_url, &client_token)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn test_connection(server_url: String, client_token: String) -> Result<String, String> {
    crate::gotify::test_connection(&server_url, &client_token)
        .await
        .map_err(|error| error.to_string())
}
