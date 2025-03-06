use std::process::Command;
use tauri::{AppHandle, Manager};

#[tauri::command]
async fn search(query: String, _app_handle: AppHandle) -> Result<(), String> {
    // let search_url = format!("{}{}", "https://www.google.com/search?q=", query);
    let search_url = format!("{}{}", "https://unduck.link?q=", query);

    // Open the URL in Chrome
    #[cfg(target_os = "windows")]
    {
        match Command::new("cmd")
            .args(["/C", "start", "chrome", &search_url])
            .spawn()
        {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to open Chrome: {}", e)),
        }
    }
}

#[tauri::command]
async fn get_suggestions(query: String) -> Result<Vec<String>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    // Google's suggestion API URL
    let url = format!(
        "https://suggestqueries.google.com/complete/search?client=chrome&q={}",
        urlencoding::encode(&query)
    );

    // Fetch suggestions
    match reqwest::get(&url).await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        // Google returns an array where the second element contains the suggestions
                        if let Some(suggestions) = data.get(1).and_then(|v| v.as_array()) {
                            let result: Vec<String> = suggestions
                                .iter()
                                .filter_map(|s| s.as_str().map(|s| s.to_string()))
                                .collect();
                            return Ok(result);
                        }
                        Ok(vec![])
                    }
                    Err(e) => Err(format!("Failed to parse response: {}", e)),
                }
            } else {
                Err(format!("Request failed with status: {}", response.status()))
            }
        }
        Err(e) => Err(format!("Failed to fetch suggestions: {}", e)),
    }
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Hide the window at startup
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();

                // Set up window event listeners for focus loss
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(focused) = event {
                        if !focused {
                            // Hide window when it loses focus
                            let _ = window_clone.hide();
                        }
                    }
                });

                // Handle Escape key in the frontend instead
                // We'll emit an event to the frontend to listen for Escape key
            }

            {
                use tauri_plugin_global_shortcut::{
                    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
                };

                let open_shortcut = Shortcut::new(Some(Modifiers::SHIFT), Code::Space);

                // Clone the app handle for use in the closure
                let app_handle = app.handle().clone();

                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(move |_app, shortcut, event| {
                            // Only run on downpress
                            if let ShortcutState::Pressed = event.state() {
                                println!("{:?}", shortcut);
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
        .invoke_handler(tauri::generate_handler![search, get_suggestions])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
