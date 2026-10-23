use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

/// Build and configure the system tray icon.
///
/// The returned handle must stay alive for the whole process: dropping a
/// `TrayIcon` removes the icon from the system tray.
pub fn build_tray(
    app_handle: &AppHandle,
) -> Result<tauri::tray::TrayIcon, Box<dyn std::error::Error>> {
    // Create tray menu items
    let show_messages = MenuItemBuilder::with_id("show_messages", "消息列表").build(app_handle)?;
    let show_settings = MenuItemBuilder::with_id("show_settings", "设置").build(app_handle)?;
    let separator = tauri::menu::PredefinedMenuItem::separator(app_handle)?;
    let quit = MenuItemBuilder::with_id("quit", "退出").build(app_handle)?;

    let menu = MenuBuilder::new(app_handle)
        .item(&show_messages)
        .item(&show_settings)
        .item(&separator)
        .item(&quit)
        .build()?;

    // Create a simple tray icon programmatically (bell shape)
    let icon = create_tray_icon();

    let tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("Nexthrum")
        .on_menu_event(move |app, event| {
            match event.id().as_ref() {
                "show_messages" => {
                    if let Some(window) = app.get_webview_window("message") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "show_settings" => {
                    // Emit event so the frontend shows settings
                    let _ = app.emit("show-settings", ());
                    if let Some(window) = app.get_webview_window("message") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                // On left click, show the message window
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("message") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app_handle)?;

    Ok(tray)
}

/// Reconcile the store's autostart entry with the persisted setting.
pub fn set_autostart(app: &AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;

    let launcher = app.autolaunch();
    let current = launcher
        .is_enabled()
        .map_err(|error| format!("could not read the launch-at-login state: {error}"))?;
    apply_autostart(
        current,
        enabled,
        || {
            launcher
                .enable()
                .map_err(|error| format!("could not enable launch at login: {error}"))
        },
        || {
            launcher
                .disable()
                .map_err(|error| format!("could not disable launch at login: {error}"))
        },
    )
}

/// Pure decision logic so the reconcile behaviour is unit-testable.
pub fn apply_autostart(
    current: bool,
    desired: bool,
    enable: impl FnOnce() -> Result<(), String>,
    disable: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    if current == desired {
        return Ok(());
    }
    if desired {
        enable()
    } else {
        disable()
    }
}

/// Load the tray icon from embedded PNG file
fn create_tray_icon() -> Image<'static> {
    let img = image::load_from_memory(include_bytes!("../icons/tray-icon.png"))
        .expect("failed to decode tray icon PNG")
        .to_rgba8();
    let (w, h) = img.dimensions();
    Image::new_owned(img.into_raw(), w, h)
}

#[cfg(test)]
mod tests {
    use super::apply_autostart;
    use std::cell::Cell;

    #[test]
    fn autostart_is_only_touched_when_the_state_differs() {
        let enable_calls = Cell::new(0);
        let disable_calls = Cell::new(0);

        // Already enabled and still enabled: no OS call at all.
        apply_autostart(
            true,
            true,
            || {
                enable_calls.set(enable_calls.get() + 1);
                Ok(())
            },
            || {
                disable_calls.set(disable_calls.get() + 1);
                Ok(())
            },
        )
        .unwrap();
        apply_autostart(
            false,
            false,
            || {
                enable_calls.set(enable_calls.get() + 1);
                Ok(())
            },
            || {
                disable_calls.set(disable_calls.get() + 1);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!((enable_calls.get(), disable_calls.get()), (0, 0));
    }

    #[test]
    fn enabling_and_disabling_call_the_matching_action_once() {
        let mut enabled = false;
        apply_autostart(
            true == false,
            true,
            || {
                enabled = true;
                Ok(())
            },
            || Ok(()),
        )
        .unwrap();
        assert!(enabled);

        let mut disabled = false;
        apply_autostart(
            true,
            false,
            || Ok(()),
            || {
                disabled = true;
                Ok(())
            },
        )
        .unwrap();
        assert!(disabled);
    }

    #[test]
    fn failures_are_propagated_to_the_caller() {
        let error =
            apply_autostart(false, true, || Err("registry locked".into()), || Ok(())).unwrap_err();
        assert!(error.contains("registry locked"));

        let error =
            apply_autostart(true, false, || Ok(()), || Err("policy denied".into())).unwrap_err();
        assert!(error.contains("policy denied"));
    }
}
