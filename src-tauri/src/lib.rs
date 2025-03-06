use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

mod bangs;
mod search;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
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
            search::get_available_bangs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
