use std::process::Command;
use tauri::{AppHandle, Manager, Window, WindowEvent};

#[tauri::command]
async fn search(query: String, _app_handle: AppHandle) -> Result<(), String> {
    let search_url = format!("{}{}", "https://www.google.com/search?q=", query);
    
    // Open the URL in Chrome
    #[cfg(target_os = "windows")]
    {
        match Command::new("cmd")
            .args(["/C", "start", "chrome", &search_url])
            .spawn() {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to open Chrome: {}", e)),
            }
    }
}

#[tauri::command]
fn toggle_window(window: Window) {
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();
    
    // Add the opener plugin
    builder = builder.plugin(tauri_plugin_opener::init());
    
    // Add the global shortcut plugin with handler
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};
        
        // Create a Shift+Space shortcut for comparison
        let shift_space = Shortcut::new(Some(Modifiers::SHIFT), Code::Space);
        
        let shortcut_plugin = tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                // Compare the shortcut directly with our shift_space shortcut
                if *shortcut == shift_space {
                    if let ShortcutState::Pressed = event.state() {
                        println!("Shift+Space pressed!");
                        if let Some(window) = app.get_webview_window("main") {
                            if let Ok(visible) = window.is_visible() {
                                if visible {
                                    println!("Hiding window");
                                    let _ = window.hide();
                                } else {
                                    println!("Showing window");
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                    }
                }
            })
            .build();
        
        builder = builder.plugin(shortcut_plugin);
    }
    
    // Continue with the builder configuration
    builder
        .setup(|app| {
            // Set up window event listeners for the main window
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::Focused(focused) = event {
                        if !focused {
                            // Hide window when it loses focus
                            let _ = window_clone.hide();
                        }
                    }
                });
                
                // Register Shift+Space shortcut to toggle window visibility
                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                {
                    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
                    
                    // Create a Shift+Space shortcut
                    let shift_space = Shortcut::new(Some(Modifiers::SHIFT), Code::Space);
                    
                    // Register the shortcut
                    if let Err(err) = app.handle().global_shortcut().register(shift_space) {
                        eprintln!("Failed to register global shortcut: {}", err);
                    } else {
                        println!("Global shortcut 'Shift+Space' registered successfully");
                    }
                }
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            search,
            toggle_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
