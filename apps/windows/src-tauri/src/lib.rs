mod commands;
mod credentials;
mod paths;
mod tray;
mod websocket;

pub use nexthrum_core::{AppSettings, GotifyMessage, MessageListItem};
pub use nexthrum_gotify as gotify;

use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;

/// State owned by the Windows/Tauri host adapter.
pub struct AppState {
    pub settings: Arc<Mutex<AppSettings>>,
    pub ws_handle: Arc<Mutex<Option<websocket::WsHandle>>>,
    pub storage_paths: nexthrum_storage::StoragePaths,
    pub connection_update: Arc<Mutex<()>>,
    pub message_update: Arc<Mutex<()>>,
}

/// Publish the authoritative unread count to the UI and the tray icon.
pub fn emit_unread(app: &tauri::AppHandle, unread: i64) {
    let _ = app.emit("unread-count-changed", unread);
    tray::update_unread(app, unread);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    env_logger::init();

    let storage_paths =
        paths::platform_default().expect("failed to resolve Nexthrum storage paths");
    let settings = credentials::load_settings(&storage_paths).unwrap_or_default();
    let app_state = AppState {
        settings: Arc::new(Mutex::new(settings.clone())),
        ws_handle: Arc::new(Mutex::new(None)),
        storage_paths: storage_paths.clone(),
        connection_update: Arc::new(Mutex::new(())),
        message_update: Arc::new(Mutex::new(())),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(Default::default(), None))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(app_state)
        .setup(move |app| {
            let message_window = tauri::WebviewWindowBuilder::new(
                app,
                "message",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("Nexthrum")
            .inner_size(800.0, 600.0)
            .min_inner_size(400.0, 300.0)
            .decorations(false)
            .shadow(true)
            .visible(!settings.start_hidden)
            .build()?;

            let app_handle_for_event = app.handle().clone();
            message_window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    let should_hide = app_handle_for_event
                        .try_state::<AppState>()
                        .map(|state| {
                            state
                                .settings
                                .try_lock()
                                .map(|settings| settings.minimize_to_tray)
                                .unwrap_or(true)
                        })
                        .unwrap_or(true);
                    if should_hide {
                        api.prevent_close();
                        if let Some(window) = app_handle_for_event.get_webview_window("message") {
                            let _ = window.hide();
                        }
                    }
                }
            });

            // The tray handle must be retained or the icon disappears immediately.
            let tray = tray::build_tray(app.handle())?;
            app.manage(tray);

            // Seed the tray badge before any message arrives.
            if let Ok(unread) = nexthrum_storage::get_unread_count(&storage_paths) {
                tray::update_unread(app.handle(), unread);
            }

            // Reconcile the OS autostart entry with the persisted preference.
            let autostart_handle = app.handle().clone();
            let autostart_enabled = settings.auto_start;
            tauri::async_runtime::spawn(async move {
                if let Err(error) = tray::set_autostart(&autostart_handle, autostart_enabled) {
                    log::warn!("Could not reconcile launch at login: {error}");
                }
            });

            if !settings.server_url.is_empty() && !settings.client_token.is_empty() {
                let handle = app.handle().clone();
                let url = settings.server_url.clone();
                let token = settings.client_token.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = websocket::connect(&handle, &url, &token).await;
                });
            }

            let handle = app.handle().clone();
            let settings_clone = settings.clone();
            let paths = storage_paths.clone();
            tauri::async_runtime::spawn(async move {
                let _ = nexthrum_storage::cleanup_old_messages(&paths, &settings_clone);
                let unread = nexthrum_storage::get_unread_count(&paths).unwrap_or(0);
                emit_unread(&handle, unread);
            });

            let handle_clone = app.handle().clone();
            let paths = storage_paths.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                    // Copy the settings out so the lock is not held across SQLite
                    // work (which would delay saving settings for that duration).
                    let snapshot = {
                        let state = handle_clone.state::<AppState>();
                        let settings = state.settings.lock().await;
                        settings.clone()
                    };
                    let _ = nexthrum_storage::cleanup_old_messages(&paths, &snapshot);
                    let unread = nexthrum_storage::get_unread_count(&paths).unwrap_or(0);
                    emit_unread(&handle_clone, unread);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::save_settings,
            commands::get_settings,
            commands::get_applications,
            commands::test_connection,
            commands::search_messages,
            commands::get_message_snapshot,
            commands::mark_message_read,
            commands::delete_message,
            commands::clear_messages,
            commands::connect_websocket,
            commands::disconnect_websocket,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
