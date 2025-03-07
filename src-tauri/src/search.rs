use crate::bangs;
use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::State;
use url::Url;

pub struct BangState {
    pub bangs: Mutex<HashMap<String, bangs::Bang>>,
}

#[tauri::command]
pub async fn get_search_suggestions(query: String) -> Result<Vec<String>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    println!("Getting suggestions: '{}'", query);

    // Google's suggestion API URL
    let url = format!(
        "https://suggestqueries.google.com/complete/search?client=chrome&q={}",
        urlencoding::encode(&query)
    );

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
                            println!("Found {} suggestions", result.len());
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

#[tauri::command]
pub async fn search(query: String, bang_state: State<'_, BangState>) -> Result<(), String> {
    // Check if the query is a URL
    let url = if is_url(&query) {
        // Ensure the URL has a scheme (http/https)
        ensure_url_scheme(query)
    } else {
        // Not a URL, process as a regular search with potential bangs
        get_bang_redirect_url(query, &bang_state)
    };

    println!("Opening URL: {}", url);

    // Spawn browser process
    #[cfg(target_os = "windows")]
    {
        open_url_without_console(&url)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Opening browser is only supported on Windows".to_string())
    }
}

#[tauri::command]
pub async fn open_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        open_url_without_console(&url)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Opening browser is only supported on Windows".to_string())
    }
}

#[cfg(target_os = "windows")]
fn open_url_without_console(url: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    // CREATE_NO_WINDOW flag to prevent console window from appearing
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    match Command::new("cmd")
        .args(["/C", "start", "chrome", url])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
    {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to open Chrome: {}", e)),
    }
}

// Function to check if a string is likely a URL
fn is_url(input: &str) -> bool {
    let input = input.trim();

    // First, check if it's a valid URL with scheme
    if let Ok(_) = Url::parse(input) {
        return true;
    }

    // Check if adding https:// prefix makes it a valid URL
    if let Ok(_) = Url::parse(&format!("https://{}", input)) {
        // Make sure it has at least one dot (for domain)
        return input.contains(".");
    }

    false
}

// Function to ensure a URL has a scheme (http/https)
fn ensure_url_scheme(url: String) -> String {
    let url = url.trim();

    // Check if it's already a valid URL with scheme
    if Url::parse(url).is_ok() {
        return url.to_string();
    }

    // Try adding https:// prefix
    let with_https = format!("https://{}", url);
    if Url::parse(&with_https).is_ok() {
        return with_https;
    }

    // If all else fails, just return the original with https:// prefix
    // The browser will handle invalid URLs
    format!("https://{}", url)
}

fn get_bang_redirect_url(query: String, bang_state: &State<BangState>) -> String {
    // Check if the query contains a bang pattern (!something)
    if let Some(bang_pos) = query.rfind('!') {
        let (search_term, bang_part) = query.split_at(bang_pos);
        let bang = &bang_part[1..]; // Remove the ! character

        // Try to find the bang in our database
        let bangs_lock = bang_state.bangs.lock().unwrap();
        if let Some(url) = bangs::get_bang_url(&bangs_lock, bang, search_term.trim()) {
            return url;
        }
    }

    // Default to Google search if no bang or unrecognized bang
    format!(
        "https://www.google.com/search?q={}",
        urlencoding::encode(&query)
    )
}

// Add a command to get all available bangs
#[tauri::command]
pub fn get_available_bangs(bang_state: State<'_, BangState>) -> Vec<(String, String)> {
    let bangs_lock = bang_state.bangs.lock().unwrap();
    bangs::get_all_bangs(&bangs_lock)
}

// Add a command to refresh bangs from DuckDuckGo
#[tauri::command]
pub async fn refresh_bangs(
    app_handle: AppHandle,
    bang_state: State<'_, BangState>,
) -> Result<(), String> {
    let bangs = bangs::load_all_bangs(&app_handle).await;

    // Update the state
    let mut bangs_lock = bang_state.bangs.lock().unwrap();
    *bangs_lock = bangs;

    Ok(())
}

// Add a command to add a custom bang
#[tauri::command]
pub fn add_custom_bang(
    app_handle: AppHandle,
    bang_state: State<'_, BangState>,
    id: String,
    name: String,
    search_url: String,
    home_url: String,
    category: String,
) -> Result<(), String> {
    let bang = bangs::Bang {
        id,
        name,
        search_url,
        home_url,
        category,
        is_custom: true,
    };

    let mut bangs_lock = bang_state.bangs.lock().unwrap();
    bangs::add_custom_bang(&app_handle, &mut bangs_lock, bang)
}

// Add a command to delete a custom bang
#[tauri::command]
pub fn delete_custom_bang(
    app_handle: AppHandle,
    bang_state: State<'_, BangState>,
    bang_id: String,
) -> Result<(), String> {
    let mut bangs_lock = bang_state.bangs.lock().unwrap();
    bangs::delete_custom_bang(&app_handle, &mut bangs_lock, &bang_id)
}
