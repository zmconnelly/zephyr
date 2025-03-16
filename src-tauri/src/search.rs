use crate::bangs::{self, Bang};
use crate::logger;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::State;
use url::Url;

pub struct BangState {
    pub bangs: Mutex<HashMap<String, Bang>>,
}

#[tauri::command]
pub async fn get_search_suggestions(query: String) -> Result<Vec<String>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    logger::info(&format!("Getting suggestions: '{}'", query));

    let url = format!(
        "https://suggestqueries.google.com/complete/search?q={}",
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
                            logger::info(&format!("Found {} suggestions", result.len()));
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
pub async fn search(
    app_handle: AppHandle,
    query: String,
    bang_state: State<'_, BangState>,
) -> Result<(), String> {
    let url = if is_url(&query) {
        ensure_url_scheme(query)
    } else {
        get_bang_redirect_url(query, &bang_state)
    };

    logger::info(&format!("Opening URL: {}", url));

    open_url(app_handle, &url)
}

#[tauri::command]
pub fn open_url(app_handle: AppHandle, url: &str) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;

    app_handle
        .opener()
        .open_url(url, None::<&str>)
        .map_err(|e| format!("Failed to open URL: {}", e))
}

fn is_url(input: &str) -> bool {
    let input = input.trim();

    if let Ok(_) = Url::parse(input) {
        return true;
    }

    if let Ok(_) = Url::parse(&format!("https://{}", input)) {
        return input.contains(".");
    }

    return false;
}

fn ensure_url_scheme(url: String) -> String {
    let url = url.trim();

    if Url::parse(url).is_ok() {
        return url.to_string();
    }

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

#[tauri::command]
pub fn get_available_bangs(bang_state: State<'_, BangState>) -> Vec<(String, String)> {
    let bangs_lock = bang_state.bangs.lock().unwrap();
    bangs::get_all_bangs(&bangs_lock)
}

#[tauri::command]
pub async fn refresh_bangs(
    app_handle: AppHandle,
    bang_state: State<'_, BangState>,
) -> Result<(), String> {
    let bangs = bangs::refresh_bangs(&app_handle).await?;

    let mut bangs_lock = bang_state.bangs.lock().unwrap();
    *bangs_lock = bangs;

    Ok(())
}

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

#[tauri::command]
pub fn delete_custom_bang(
    app_handle: AppHandle,
    bang_state: State<'_, BangState>,
    bang_id: String,
) -> Result<(), String> {
    let mut bangs_lock = bang_state.bangs.lock().unwrap();
    bangs::delete_custom_bang(&app_handle, &mut bangs_lock, &bang_id)
}

#[tauri::command]
pub async fn clear_bangs_cache(
    app_handle: AppHandle,
    bang_state: State<'_, BangState>,
) -> Result<(), String> {
    let bangs = bangs::refresh_bangs(&app_handle).await?;

    let mut bangs_lock = bang_state.bangs.lock().unwrap();
    *bangs_lock = bangs;

    Ok(())
}
