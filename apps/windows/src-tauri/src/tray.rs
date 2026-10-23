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
    image_from_bytes(include_bytes!("../icons/tray-icon.png"), false)
}

fn image_from_bytes(bytes: &[u8], with_badge: bool) -> Image<'static> {
    let mut img = image::load_from_memory(bytes)
        .expect("failed to decode tray icon PNG")
        .to_rgba8();
    if with_badge {
        draw_badge(&mut img);
    }
    let (w, h) = img.dimensions();
    Image::new_owned(img.into_raw(), w, h)
}

/// Paint the unread dot in the top-right corner.
fn draw_badge(img: &mut image::RgbaImage) {
    let (width, _height) = img.dimensions();
    let radius: i32 = (width as i32 / 6).max(3);
    let center_x = width as i32 - radius - 2;
    let center_y = radius + 2;
    let border = 1;

    for y in (center_y - radius - border)..=(center_y + radius + border) {
        for x in (center_x - radius - border)..=(center_x + radius + border) {
            if x < 0 || y < 0 || x >= width as i32 || y >= img.dimensions().1 as i32 {
                continue;
            }
            let distance = ((x - center_x).pow(2) + (y - center_y).pow(2)) as f64;
            let inner = ((radius as f64) - 0.5).powi(2);
            let outer = ((radius + border) as f64 + 0.5).powi(2);
            if distance <= inner {
                // Danger red matches --color-danger in the UI.
                img.put_pixel(x as u32, y as u32, image::Rgba([239, 68, 68, 255]));
            } else if distance <= outer {
                // Thin light ring keeps the dot visible on dark icons.
                img.put_pixel(x as u32, y as u32, image::Rgba([255, 255, 255, 220]));
            }
        }
    }
}

/// Tooltip text shown while hovering the tray icon.
pub fn tooltip_text(unread: i64) -> String {
    if unread > 0 {
        format!("Nexthrum — {unread} 条未读")
    } else {
        "Nexthrum".to_string()
    }
}

/// Reflect the current unread count on the tray icon.
pub fn update_unread(app: &AppHandle, unread: i64) {
    let Some(tray) = app.try_state::<tauri::tray::TrayIcon>() else {
        return;
    };
    let icon = image_from_bytes(include_bytes!("../icons/tray-icon.png"), unread > 0);
    if let Err(error) = tray.set_icon(Some(icon)) {
        log::warn!("Failed to update the tray icon: {error}");
    }
    if let Err(error) = tray.set_tooltip(Some(tooltip_text(unread))) {
        log::warn!("Failed to update the tray tooltip: {error}");
    }
}

#[cfg(test)]
mod unread_tests {
    use super::{draw_badge, tooltip_text};

    #[test]
    fn tooltip_reports_the_unread_count() {
        assert_eq!(tooltip_text(0), "Nexthrum");
        assert_eq!(tooltip_text(1), "Nexthrum — 1 条未读");
        assert_eq!(tooltip_text(12), "Nexthrum — 12 条未读");
    }

    #[test]
    fn badge_paints_the_top_right_corner_only() {
        let mut image = image::RgbaImage::from_pixel(32, 32, image::Rgba([0, 0, 0, 0]));
        draw_badge(&mut image);

        // Top-right corner is painted…
        let corner = image.get_pixel(28, 6);
        assert!(corner[3] > 0, "expected a badge pixel, got {corner:?}");
        // …while the opposite corner stays untouched.
        let far = image.get_pixel(4, 28);
        assert_eq!(far[3], 0, "bottom-left must stay transparent");
    }
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
