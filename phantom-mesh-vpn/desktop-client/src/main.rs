//! PhantomMesh VPN Desktop Client
//!
//! A Tauri-based desktop application providing secure VPN connectivity
//! with AI-powered threat detection and autonomous protection.
//!
//! Copyright © 2026 PhantomMesh. All rights reserved.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod error;
mod state;
mod tray;
mod vpn;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use state::AppState;
use tray::setup_system_tray;

fn main() {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_target(true)
        .with_thread_names(true)
        .pretty()
        .init();

    info!("🚀 PhantomMesh VPN Desktop Client starting...");

    // Build and run the Tauri application
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            info!("Single instance triggered: {:?} {:?}", argv, cwd);
            // Focus the main window when another instance is launched
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            info!("Setting up PhantomMesh VPN client...");

            // Initialize application state
            let state = Arc::new(RwLock::new(AppState::new()));
            app.manage(state);

            // Setup system tray
            setup_system_tray(app)?;

            // Get main window and configure it
            if let Some(window) = app.get_webview_window("main") {
                // Enable devtools in debug mode
                #[cfg(debug_assertions)]
                window.open_devtools();

                info!("Main window initialized successfully");
            }

            info!("✅ PhantomMesh VPN client setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Connection commands
            commands::connect,
            commands::disconnect,
            commands::quick_connect,
            commands::get_connection_status,
            // Server commands
            commands::get_servers,
            commands::get_server_load,
            commands::ping_server,
            // Settings commands
            commands::get_settings,
            commands::update_settings,
            commands::toggle_kill_switch,
            commands::toggle_auto_connect,
            // Statistics commands
            commands::get_connection_stats,
            commands::get_bandwidth_usage,
            // Account commands
            commands::get_account_info,
            // System commands
            commands::check_updates,
            commands::get_app_version,
            commands::open_logs_folder,
        ])
        .on_window_event(|window, event| {
            use tauri::WindowEvent;
            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    // Minimize to tray instead of closing
                    let _ = window.hide();
                    api.prevent_close();
                    info!("Window hidden to system tray");
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("Failed to run PhantomMesh VPN application");
}
