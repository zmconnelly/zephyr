#![allow(deprecated)]

use std::env;
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};
use system_tray::initialize_tray;
use tauri::Manager;

use windows_key_listener::KeyListener;

mod bangs;
mod logger;
mod search;
mod startup;
mod system_tray;
mod updater;

pub fn run() {
    logger::info(&format!("Starting Zephyr v{}", env!("CARGO_PKG_VERSION")));

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let key_listener = KeyListener::new();

            let app_handle = app.handle().clone();

            key_listener
                .listen(
                    "Ctrl + Space",
                    std::time::Duration::from_millis(200),
                    Arc::new(move || {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            if !window.is_visible().unwrap_or(false) {
                                let _ = window.show();
                                let _ = window.set_focus();
                                return true;
                            }
                        }

                        return false;
                    }),
                )
                .expect("Failed to listen to key chord");

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if updater::is_dev_mode() {
                    logger::info("Skipping update check in development mode");
                    return;
                }

                // Wait a bit before checking for updates
                std::thread::sleep(std::time::Duration::from_secs(5));

                if let Err(e) = updater::check_for_updates(app_handle).await {
                    logger::error(&format!("Update check failed: {}", e));
                }
            });

            initialize_tray(&app);

            // Initialize bangs
            let app_handle = app.handle().clone();

            // Create initial bang state with fallback bangs
            let initial_bangs = HashMap::new();

            // Spawn a task to load bangs asynchronously
            tauri::async_runtime::spawn(async move {
                let loaded_bangs = bangs::load_all_bangs(&app_handle).await;

                // Update the state with loaded bangs
                if let Some(state) = app_handle.try_state::<search::BangState>() {
                    let mut bangs_lock = state.bangs.lock().unwrap();
                    *bangs_lock = loaded_bangs;
                }
            });

            // Register the bang state
            app.manage(search::BangState {
                bangs: Mutex::new(initial_bangs),
            });

            // Hide the window at startup
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();

                // Hide window when it loses focus
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(focused) = event {
                        if !focused {
                            window_clone.hide().unwrap();
                        }
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            search::get_search_suggestions,
            search::search,
            search::get_available_bangs,
            search::refresh_bangs,
            search::add_custom_bang,
            search::delete_custom_bang,
            search::open_url,
            search::clear_bangs_cache,
            startup::get_startup_status,
            startup::toggle_run_at_startup,
            logger::log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
