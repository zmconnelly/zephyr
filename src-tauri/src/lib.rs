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

pub fn run2() {
    tauri::Builder::default()
        .setup(|app| {
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
        .invoke_handler(tauri::generate_handler![search])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
