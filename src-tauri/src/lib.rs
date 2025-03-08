#![allow(deprecated)]

use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use winreg::enums::*;
use winreg::RegKey;

mod bangs;
mod search;

// Add a new module for startup settings
mod startup {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::env;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct StartupStatus {
        pub enabled: bool,
    }

    #[tauri::command]
    pub fn get_startup_status() -> Result<StartupStatus, String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = match hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run") {
            Ok(key) => key,
            Err(_) => return Ok(StartupStatus { enabled: false }),
        };

        // Check if our app is in the startup registry
        let is_enabled = run_key.get_value::<String, _>("Zephyr").is_ok();

        Ok(StartupStatus {
            enabled: is_enabled,
        })
    }

    #[tauri::command]
    pub fn toggle_run_at_startup(enable: bool) -> Result<StartupStatus, String> {
        println!("Toggling run at startup: {}", enable);

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        // Open or create the Run registry key
        let run_key_result = hkcu.open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            KEY_WRITE,
        );

        let run_key = match run_key_result {
            Ok(key) => key,
            Err(_) => {
                // Create the key if it doesn't exist
                let (key, _) = hkcu
                    .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
                    .map_err(|e| e.to_string())?;
                key
            }
        };

        if enable {
            // Get path to the executable using env::current_exe()
            let exe_path = env::current_exe()
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .to_string();

            run_key
                .set_value("Zephyr", &exe_path)
                .map_err(|e| e.to_string())?;
        } else {
            // Remove from startup
            let _ = run_key.delete_value("Zephyr"); // Ignore errors if value doesn't exist
        }

        Ok(StartupStatus { enabled: enable })
    }
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;

            TrayIconBuilder::new()
                .menu(&menu)
                .menu_on_left_click(true)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        println!("Quitting application...");
                        app.exit(0);
                    }
                    _ => {
                        println!("Menu item {:?} not handled", event.id);
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

            {
                let open_shortcut = Shortcut::new(Some(Modifiers::SHIFT), Code::Space);

                let app_handle = app.handle().clone();

                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(move |_app, _shortcut, event| {
                            if let ShortcutState::Pressed = event.state() {
                                if let Some(window) = app_handle.get_webview_window("main") {
                                    if !window.is_visible().unwrap_or(false) {
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                    }
                                }
                            }
                        })
                        .build(),
                )?;

                app.global_shortcut().register(open_shortcut)?;
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
            startup::get_startup_status,
            startup::toggle_run_at_startup,
            log_to_console
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Add a new function to handle console logs from the frontend
#[tauri::command]
fn log_to_console(level: &str, message: &str) {
    match level {
        "error" => eprintln!("\x1b[31m[WebView] {}\x1b[0m", message), // Red for errors
        "warn" => println!("\x1b[33m[WebView] {}\x1b[0m", message),   // Yellow for warnings
        "info" => println!("\x1b[32m[WebView] {}\x1b[0m", message),   // Green for info
        "debug" => println!("\x1b[36m[WebView] {}\x1b[0m", message),  // Cyan for debug
        _ => println!("[WebView] {}", message),                       // Default with no color
    }
}
