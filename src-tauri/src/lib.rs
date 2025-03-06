#![allow(deprecated)]

use std::collections::HashMap;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

mod bangs;
mod search;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
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
                                    if window.is_visible().unwrap_or(false) {
                                        let _ = window.hide();
                                    } else {
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
