use std::process::Command;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State, Window, WindowEvent};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchConfig {
    engine_url: String,
    engine_name: String,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            engine_url: String::from("https://www.google.com/search?q="),
            engine_name: String::from("Google"),
        }
    }
}

// Available search engines
pub fn available_engines() -> Vec<SearchConfig> {
    vec![
        SearchConfig {
            engine_name: String::from("Google"),
            engine_url: String::from("https://www.google.com/search?q="),
        },
        SearchConfig {
            engine_name: String::from("Bing"),
            engine_url: String::from("https://www.bing.com/search?q="),
        },
        SearchConfig {
            engine_name: String::from("DuckDuckGo"),
            engine_url: String::from("https://duckduckgo.com/?q="),
        },
        SearchConfig {
            engine_name: String::from("Yahoo"),
            engine_url: String::from("https://search.yahoo.com/search?p="),
        },
    ]
}

// State to hold the current search engine
pub struct SearchState(pub Mutex<SearchConfig>);

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn search(
    query: String,
    _app_handle: AppHandle,
    state: State<'_, SearchState>,
) -> Result<(), String> {
    // Get the config from state
    let config = state.0.lock().unwrap().clone();
    
    // Build the search URL
    let search_url = format!("{}{}", config.engine_url, query);
    
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
fn get_current_engine(state: State<'_, SearchState>) -> SearchConfig {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
fn get_available_engines() -> Vec<SearchConfig> {
    available_engines()
}

#[tauri::command]
fn set_search_engine(
    engine_name: String,
    state: State<'_, SearchState>,
) -> Result<SearchConfig, String> {
    let engines = available_engines();
    
    if let Some(engine) = engines.iter().find(|e| e.engine_name == engine_name) {
        let mut current = state.0.lock().unwrap();
        *current = engine.clone();
        Ok(current.clone())
    } else {
        Err(format!("Search engine '{}' not found", engine_name))
    }
}

// Toggle window visibility
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
        .manage(SearchState(Mutex::new(SearchConfig::default())))
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
            greet,
            search,
            get_current_engine,
            get_available_engines,
            set_search_engine,
            toggle_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
