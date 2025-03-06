use crate::bangs;
use reqwest::Client;
use std::process::Command;
use tauri::AppHandle;

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

#[tauri::command]
pub async fn search(query: String, _app_handle: AppHandle) -> Result<(), String> {
    let url = get_bang_redirect_url(query);

    // Spawn browser process
    #[cfg(target_os = "windows")]
    {
        match Command::new("cmd")
            .args(["/C", "start", "chrome", &url])
            .spawn()
        {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to open Chrome: {}", e)),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Opening browser is only supported on Windows".to_string())
    }
}

fn get_bang_redirect_url(query: String) -> String {
    // Check if the query contains a bang pattern (!something)
    if let Some(bang_pos) = query.rfind('!') {
        let (search_term, bang_part) = query.split_at(bang_pos);
        let bang = &bang_part[1..]; // Remove the ! character

        // Try to find the bang in our database
        if let Some(url) = bangs::get_bang_url(bang, search_term.trim()) {
            return url;
        }
    }

    // Default to DuckDuckGo if no bang or unrecognized bang
    format!("https://duckduckgo.com/?q={}", urlencoding::encode(&query))
}

// Add a command to get all available bangs
#[tauri::command]
pub fn get_available_bangs() -> Vec<(String, String)> {
    bangs::get_all_bangs()
}
