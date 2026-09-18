// tray.rs — System Tray / Menu Bar implementation for macOS, Windows, and Linux
// Enables Atena Studio to run seamlessly in the background with quick controls and status indications.

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Wry,
};

pub const TRAY_ID: &str = "atena-main-tray";

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Build Menu Items
    let title_item = MenuItem::with_id(app, "tray_title", "● Atena Studio", false, None::<&str>)?;
    let status_sep = PredefinedMenuItem::separator(app)?;
    
    let show_item = MenuItem::with_id(app, "tray_show", "Show Atena Studio", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app, "tray_hide", "Hide to Tray", true, None::<&str>)?;
    let action_sep = PredefinedMenuItem::separator(app)?;

    let quit_item = MenuItem::with_id(app, "tray_quit", "Quit Atena Studio", true, None::<&str>)?;

    // 2. Compose Menu
    let menu = Menu::with_items(
        app,
        &[
            &title_item,
            &status_sep,
            &show_item,
            &hide_item,
            &action_sep,
            &quit_item,
        ],
    )?;

    // 3. Resolve tray icon: macOS uses a monochrome template icon that automatically
    // adapts to light/dark themes (white in dark mode, black in light mode).
    #[cfg(target_os = "macos")]
    let tray_icon = {
        let icon_bytes = include_bytes!("../icons/tray-icon.png");
        tauri::image::Image::from_bytes(icon_bytes)?
    };

    #[cfg(not(target_os = "macos"))]
    let tray_icon = if let Some(icon) = app.default_window_icon() {
        icon.clone()
    } else {
        let icon_bytes = include_bytes!("../icons/32x32.png");
        tauri::image::Image::from_bytes(icon_bytes)?
    };

    // 4. Build and register Tray
    let _tray = TrayIconBuilder::<Wry>::with_id(TRAY_ID)
        .icon(tray_icon)
        .icon_as_template(true)
        .tooltip("Atena Studio - AI Cognitive Platform")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "tray_show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }
            "tray_hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            "tray_quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if let Ok(visible) = window.is_visible() {
                        if visible {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                }
            }
        })
        .build(app)?;

    log::info!("🚀 Native System Tray initialized successfully.");
    Ok(())
}
