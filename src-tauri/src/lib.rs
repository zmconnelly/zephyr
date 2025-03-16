#![allow(deprecated)]

use std::env;
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
use windows_key_listener::KeyListener;

mod bangs;
mod logger;
mod search;
mod startup;
mod updater;

pub fn run() {
    logger::info(&format!("Starting Zephyr v{}", env!("CARGO_PKG_VERSION")));

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let key_listener = KeyListener::new();

            let app_handle = app.handle().clone();

            key_listener
                .listen(
                    "Ctrl + Space",
                    std::time::Duration::from_millis(200),
                    Arc::new(move || {
                        logger::info("Ctrl + Space key chord pressed");

                        // return false;

                        if let Some(window) = app_handle.get_webview_window("main") {
                            if !window.is_visible().unwrap_or(false) {
                                let _ = window.show();
                                let _ = window.set_focus();
                                return true;
                            }
                        }
                        // Return false to allow the event to propagate, true to block it
                        return false;
                    }),
                )
                .expect("Failed to listen to key chord");
            // Check for updates on startup
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // Wait a bit before checking for updates
                std::thread::sleep(std::time::Duration::from_secs(5));

                if let Err(e) = updater::check_for_updates(app_handle).await {
                    logger::error(&format!("Update check failed: {}", e));
                }
            });

            let version = app.package_info().version.to_string();

            // Create tray menu items
            let version_item = MenuItem::with_id(
                &app.handle().clone(),
                "version",
                format!("Version: {}", version),
                false,
                None::<&str>,
            )
            .unwrap();

            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            // Add log menu items
            let open_logs_item = MenuItem::with_id(
                &app.handle().clone(),
                "open_logs",
                "Open Logs Folder",
                true,
                None::<&str>,
            )?;

            // Add refresh bangs menu item
            let refresh_bangs_item = MenuItem::with_id(
                &app.handle().clone(),
                "refresh_bangs",
                "Refresh Bangs",
                true,
                None::<&str>,
            )?;

            // Create menu with all items
            let menu = Menu::with_items(
                app,
                &[
                    &version_item,
                    &open_logs_item,
                    &refresh_bangs_item,
                    &quit_item,
                ],
            )?;

            TrayIconBuilder::new()
                .menu(&menu)
                .menu_on_left_click(true)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        logger::info("Quitting application...");
                        app.exit(0);
                    }
                    "version" => {
                        logger::info(&format!("Version: {}", env!("CARGO_PKG_VERSION")));
                        // Check for updates when version is clicked
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(e) = updater::check_for_updates(app_handle).await {
                                logger::error(&format!("Update check failed: {}", e));
                            }
                        });
                    }
                    "open_logs" => {
                        logger::info("Opening logs folder...");
                        if let Err(e) = logger::open_log_directory(app.clone()) {
                            logger::error(&format!("Failed to open logs folder: {}", e));
                        }
                    }
                    "refresh_bangs" => {
                        logger::info("Refreshing bangs cache...");
                        let app_handle_clone = app.clone();
                        tauri::async_runtime::spawn(async move {
                            match bangs::refresh_bangs(&app_handle_clone).await {
                                Ok(fresh_bangs) => {
                                    if let Some(state) =
                                        app_handle_clone.try_state::<search::BangState>()
                                    {
                                        let mut bangs_lock = state.bangs.lock().unwrap();
                                        *bangs_lock = fresh_bangs;
                                    }
                                }
                                Err(e) => {
                                    logger::error(&format!("Failed to refresh bangs: {}", e));
                                }
                            }
                        });
                    }
                    _ => {
                        logger::debug(&format!("Menu item {:?} not handled", event.id));
                    }
                })
                .build(app)?;

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
            updater::check_for_updates,
            updater::set_github_token,
            updater::get_github_token,
            updater::update_github_url,
            updater::test_github_token,
            logger::log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
