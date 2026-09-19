// tray.rs — System Tray / Menu Bar implementation for macOS, Windows, and Linux
// Enables Atena Studio to run seamlessly in the background with quick controls and status indications.

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Wry,
};

pub const TRAY_ID: &str = "atena-main-tray";

pub struct TrayLabels {
    pub title: &'static str,
    pub show: &'static str,
    pub hide: &'static str,
    pub quit: &'static str,
    pub tooltip: &'static str,
}

pub fn get_tray_labels(locale: &str) -> TrayLabels {
    match locale {
        "pt-BR" | "pt" => TrayLabels {
            title: "Atena Studio",
            show: "Mostrar Atena Studio",
            hide: "Ocultar na Bandeja",
            quit: "Encerrar Atena Studio",
            tooltip: "Atena Studio - Plataforma Cognitiva de IA",
        },
        "es" => TrayLabels {
            title: "Atena Studio",
            show: "Mostrar Atena Studio",
            hide: "Ocultar en la bandeja",
            quit: "Salir de Atena Studio",
            tooltip: "Atena Studio - Plataforma Cognitiva de IA",
        },
        "zh-CN" | "zh" => TrayLabels {
            title: "Atena Studio",
            show: "显示 Atena Studio",
            hide: "隐藏到系统托盘",
            quit: "退出 Atena Studio",
            tooltip: "Atena Studio - 人工智能认知平台",
        },
        "ru" => TrayLabels {
            title: "Atena Studio",
            show: "Показать Atena Studio",
            hide: "Скрыть в трей",
            quit: "Выйти из Atena Studio",
            tooltip: "Atena Studio - Когнитивная платформа ИИ",
        },
        _ => TrayLabels {
            title: "Atena Studio",
            show: "Show Atena Studio",
            hide: "Hide to Tray",
            quit: "Quit Atena Studio",
            tooltip: "Atena Studio - AI Cognitive Platform",
        },
    }
}

pub fn build_tray_menu(app: &AppHandle, locale: &str) -> Result<Menu<Wry>, Box<dyn std::error::Error>> {
    let labels = get_tray_labels(locale);
    let title_item = MenuItem::with_id(app, "tray_title", labels.title, false, None::<&str>)?;
    let status_sep = PredefinedMenuItem::separator(app)?;
    
    let show_item = MenuItem::with_id(app, "tray_show", labels.show, true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app, "tray_hide", labels.hide, true, None::<&str>)?;
    let action_sep = PredefinedMenuItem::separator(app)?;

    let quit_item = MenuItem::with_id(app, "tray_quit", labels.quit, true, None::<&str>)?;

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
    Ok(menu)
}

pub fn update_tray_locale(app: &AppHandle, locale: &str) -> Result<(), Box<dyn std::error::Error>> {
    let labels = get_tray_labels(locale);
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let menu = build_tray_menu(app, locale)?;
        let _ = tray.set_menu(Some(menu));
        let _ = tray.set_tooltip(Some(labels.tooltip));
        log::info!("🔄 System Tray updated with locale: {}", locale);
    }
    Ok(())
}

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Resolve configured locale
    let cfg = crate::core::config::AppConfig::load();
    let locale = cfg.language.as_str();
    let labels = get_tray_labels(locale);

    // 2. Build Menu Items with active locale
    let menu = build_tray_menu(app, locale)?;

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
        .tooltip(labels.tooltip)
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

    log::info!("🚀 Native System Tray initialized successfully with locale: {}", locale);
    Ok(())
}
