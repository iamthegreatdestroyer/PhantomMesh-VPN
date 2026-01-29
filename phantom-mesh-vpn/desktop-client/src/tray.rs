//! System tray implementation for PhantomMesh VPN

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Manager,
};
use tracing::info;

/// Setup the system tray with menu
pub fn setup_system_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting up system tray...");

    // Create menu items
    let show = MenuItem::with_id(app, "show", "Show PhantomMesh", true, None::<&str>)?;
    let connect = MenuItem::with_id(app, "quick_connect", "Quick Connect", true, None::<&str>)?;
    let disconnect = MenuItem::with_id(app, "disconnect", "Disconnect", true, None::<&str>)?;
    let separator1 = PredefinedMenuItem::separator(app)?;

    // Server submenu
    let server_us = MenuItem::with_id(app, "server_us", "🇺🇸 United States", true, None::<&str>)?;
    let server_uk = MenuItem::with_id(app, "server_uk", "🇬🇧 United Kingdom", true, None::<&str>)?;
    let server_de = MenuItem::with_id(app, "server_de", "🇩🇪 Germany", true, None::<&str>)?;
    let server_nl = MenuItem::with_id(app, "server_nl", "🇳🇱 Netherlands", true, None::<&str>)?;
    let server_jp = MenuItem::with_id(app, "server_jp", "🇯🇵 Japan", true, None::<&str>)?;
    let server_menu = Submenu::with_items(
        app,
        "Connect to...",
        true,
        &[&server_us, &server_uk, &server_de, &server_nl, &server_jp],
    )?;

    let separator2 = PredefinedMenuItem::separator(app)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let about = MenuItem::with_id(app, "about", "About", true, None::<&str>)?;
    let separator3 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit PhantomMesh", true, None::<&str>)?;

    // Build menu
    let menu = Menu::with_items(
        app,
        &[
            &show,
            &separator1,
            &connect,
            &disconnect,
            &server_menu,
            &separator2,
            &settings,
            &about,
            &separator3,
            &quit,
        ],
    )?;

    // Build tray icon
    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("PhantomMesh VPN - Disconnected")
        .on_menu_event(|app, event| {
            info!("Tray menu event: {:?}", event.id());

            match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.unminimize();
                    }
                }
                "quick_connect" => {
                    info!("Quick connect from tray");
                    // Emit event to frontend
                    let _ = app.emit("tray-quick-connect", ());
                }
                "disconnect" => {
                    info!("Disconnect from tray");
                    let _ = app.emit("tray-disconnect", ());
                }
                "server_us" => {
                    let _ = app.emit("tray-connect", "us-east-1");
                }
                "server_uk" => {
                    let _ = app.emit("tray-connect", "uk-london-1");
                }
                "server_de" => {
                    let _ = app.emit("tray-connect", "de-frankfurt-1");
                }
                "server_nl" => {
                    let _ = app.emit("tray-connect", "nl-amsterdam-1");
                }
                "server_jp" => {
                    let _ = app.emit("tray-connect", "jp-tokyo-1");
                }
                "settings" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                    let _ = app.emit("navigate", "/settings");
                }
                "about" => {
                    let _ = app.emit("navigate", "/about");
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    info!("Quitting PhantomMesh VPN");
                    std::process::exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } => {
                    // Show window on left click
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.unminimize();
                    }
                }
                TrayIconEvent::DoubleClick { .. } => {
                    // Also show window on double click
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                _ => {}
            }
        })
        .build(app)?;

    info!("System tray setup complete");
    Ok(())
}

/// Update tray icon based on connection state
pub fn update_tray_icon(app: &tauri::AppHandle, connected: bool, server_name: Option<&str>) {
    // In a full implementation, we would:
    // 1. Change the tray icon to show connected/disconnected state
    // 2. Update the tooltip with connection info
    // 3. Enable/disable appropriate menu items

    let tooltip = if connected {
        format!(
            "PhantomMesh VPN - Connected to {}",
            server_name.unwrap_or("Unknown")
        )
    } else {
        "PhantomMesh VPN - Disconnected".to_string()
    };

    info!("Tray tooltip updated: {}", tooltip);
}
