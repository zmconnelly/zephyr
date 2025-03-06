use chrono::{DateTime, Duration, Utc};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bang {
    pub id: String,
    pub name: String,
    pub search_url: String,
    pub home_url: String,
    pub category: String,
    pub is_custom: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct BangCache {
    bangs: HashMap<String, Bang>,
    #[serde(with = "timestamp_seconds")]
    last_updated: DateTime<Utc>,
}

// Serialization helper for DateTime<Utc>
mod timestamp_seconds {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(dt.timestamp())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = i64::deserialize(deserializer)?;
        Ok(Utc.timestamp_opt(timestamp, 0).unwrap())
    }
}

// A small set of fallback bangs in case we can't fetch or load any
lazy_static! {
    static ref FALLBACK_BANGS: HashMap<String, Bang> = {
        let mut m = HashMap::new();

        // Google
        m.insert("g".to_string(), Bang {
            id: "g".to_string(),
            name: "Google".to_string(),
            search_url: "https://www.google.com/search?q={{qe}}".to_string(),
            home_url: "https://www.google.com".to_string(),
            category: "Search".to_string(),
            is_custom: false,
        });

        // Wikipedia
        m.insert("w".to_string(), Bang {
            id: "w".to_string(),
            name: "Wikipedia".to_string(),
            search_url: "https://en.wikipedia.org/wiki/Special:Search?search={{qe}}".to_string(),
            home_url: "https://en.wikipedia.org".to_string(),
            category: "Reference".to_string(),
            is_custom: false,
        });

        // YouTube
        m.insert("yt".to_string(), Bang {
            id: "yt".to_string(),
            name: "YouTube".to_string(),
            search_url: "https://www.youtube.com/results?search_query={{qe}}".to_string(),
            home_url: "https://www.youtube.com".to_string(),
            category: "Video".to_string(),
            is_custom: false,
        });

        // GitHub
        m.insert("gh".to_string(), Bang {
            id: "gh".to_string(),
            name: "GitHub".to_string(),
            search_url: "https://github.com/search?q={{qe}}".to_string(),
            home_url: "https://github.com".to_string(),
            category: "Development".to_string(),
            is_custom: false,
        });

        // DuckDuckGo
        m.insert("ddg".to_string(), Bang {
            id: "ddg".to_string(),
            name: "DuckDuckGo".to_string(),
            search_url: "https://duckduckgo.com/?q={{qe}}".to_string(),
            home_url: "https://duckduckgo.com".to_string(),
            category: "Search".to_string(),
            is_custom: false,
        });

        m
    };
}

// Get paths for cache and user settings
fn get_cache_path(_app_handle: &AppHandle) -> PathBuf {
    let mut path = if let Ok(cache_dir) = env::var("APPDATA") {
        // Windows
        PathBuf::from(cache_dir)
    } else if let Ok(home) = env::var("HOME") {
        // macOS/Linux
        let mut p = PathBuf::from(home);
        if cfg!(target_os = "macos") {
            p.push("Library/Caches");
        } else {
            p.push(".cache");
        }
        p
    } else {
        PathBuf::from("./")
    };

    path.push("zephyr");
    path.push("bangs_cache.json");

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    path
}

fn get_settings_path(_app_handle: &AppHandle) -> PathBuf {
    let mut path = if let Ok(config_dir) = env::var("APPDATA") {
        // Windows
        PathBuf::from(config_dir)
    } else if let Ok(home) = env::var("HOME") {
        // macOS/Linux
        let mut p = PathBuf::from(home);
        if cfg!(target_os = "macos") {
            p.push("Library/Application Support");
        } else {
            p.push(".config");
        }
        p
    } else {
        PathBuf::from("./")
    };

    path.push("zephyr");
    path.push("user_bangs.json");

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    path
}

// Load bangs from cache
fn load_cache(app_handle: &AppHandle) -> Option<BangCache> {
    let cache_path = get_cache_path(app_handle);
    if cache_path.exists() {
        if let Ok(file) = fs::read_to_string(&cache_path) {
            if let Ok(cache) = serde_json::from_str::<BangCache>(&file) {
                return Some(cache);
            }
        }
    }
    None
}

// Save bangs to cache
fn save_cache(app_handle: &AppHandle, bangs: &HashMap<String, Bang>) -> Result<(), String> {
    let cache_path = get_cache_path(app_handle);

    // Create directory if it doesn't exist
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let cache = BangCache {
        bangs: bangs.clone(),
        last_updated: Utc::now(),
    };

    let json = serde_json::to_string_pretty(&cache).map_err(|e| e.to_string())?;
    fs::write(&cache_path, json).map_err(|e| e.to_string())
}

// Load user custom bangs
fn load_user_bangs(app_handle: &AppHandle) -> HashMap<String, Bang> {
    let settings_path = get_settings_path(app_handle);
    if settings_path.exists() {
        if let Ok(file) = fs::read_to_string(&settings_path) {
            if let Ok(user_bangs) = serde_json::from_str::<HashMap<String, Bang>>(&file) {
                return user_bangs;
            }
        }
    }
    HashMap::new()
}

// Save user custom bangs
pub fn save_user_bangs(
    app_handle: &AppHandle,
    bangs: &HashMap<String, Bang>,
) -> Result<(), String> {
    let settings_path = get_settings_path(app_handle);

    // Create directory if it doesn't exist
    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let json = serde_json::to_string_pretty(&bangs).map_err(|e| e.to_string())?;
    fs::write(&settings_path, json).map_err(|e| e.to_string())
}

// Parse DuckDuckGo bang.js file
fn parse_duckduckgo_bangs(js_content: &str) -> Result<HashMap<String, Bang>, String> {
    let mut bangs = HashMap::new();

    // The bang.js file is a JavaScript array of objects
    // We need to extract the relevant data

    // This is a simplified parser - in a real implementation, you'd want to use
    // a proper JavaScript parser or regex with capture groups

    // Example entry in bang.js:
    // {"c":"Tech","d":"www.01net.com","r":4,"s":"01net","sc":"Downloads (apps)","t":"01net","u":"http://www.01net.com/recherche/recherche.php?searchstring={{{s}}}&chaine=home"}

    // Extract JSON objects from the array
    let js_content = js_content.trim();
    if !js_content.starts_with('[') || !js_content.ends_with(']') {
        return Err("Invalid bang.js format".to_string());
    }

    let content = &js_content[1..js_content.len() - 1];

    // Split by },{
    for item in content.split("},{") {
        let mut item = item.to_string();
        if !item.starts_with('{') {
            item = format!("{{{}", item);
        }
        if !item.ends_with('}') {
            item = format!("{}}}", item);
        }

        // Parse the JSON object
        if let Ok(bang_data) = serde_json::from_str::<serde_json::Value>(&item) {
            // Extract the relevant fields
            if let (
                Some(category),
                Some(domain),
                Some(name),
                Some(subcategory),
                Some(trigger),
                Some(url),
            ) = (
                bang_data.get("c").and_then(|v| v.as_str()),
                bang_data.get("d").and_then(|v| v.as_str()),
                bang_data.get("s").and_then(|v| v.as_str()),
                bang_data.get("sc").and_then(|v| v.as_str()),
                bang_data.get("t").and_then(|v| v.as_str()),
                bang_data.get("u").and_then(|v| v.as_str()),
            ) {
                let bang = Bang {
                    id: trigger.to_string(),
                    name: name.to_string(),
                    search_url: url.to_string(),
                    home_url: format!("https://{}", domain),
                    category: format!("{} - {}", category, subcategory),
                    is_custom: false,
                };

                bangs.insert(trigger.to_string(), bang);
            }
        }
    }

    Ok(bangs)
}

// Fetch DuckDuckGo bangs
async fn fetch_duckduckgo_bangs() -> Result<HashMap<String, Bang>, String> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://duckduckgo.com/bang.js")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch bangs: {}", e))?;

    if response.status().is_success() {
        let js_content = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        parse_duckduckgo_bangs(&js_content)
    } else {
        Err(format!("Failed to fetch bangs: HTTP {}", response.status()))
    }
}

// Load all bangs (DuckDuckGo + user custom)
pub async fn load_all_bangs(app_handle: &AppHandle) -> HashMap<String, Bang> {
    // First, try to load from cache
    let mut should_update = false;
    let mut bangs = if let Some(cache) = load_cache(app_handle) {
        // Check if cache is older than 7 days
        let now = Utc::now();
        if now - cache.last_updated > Duration::days(7) {
            should_update = true;
        }
        cache.bangs
    } else {
        // No cache, need to fetch
        should_update = true;
        HashMap::new()
    };

    // If cache is empty or old, try to update it
    if should_update {
        match fetch_duckduckgo_bangs().await {
            Ok(fetched_bangs) => {
                bangs = fetched_bangs;
                // Save to cache in the background
                let app_handle_clone = app_handle.clone();
                let bangs_clone = bangs.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = save_cache(&app_handle_clone, &bangs_clone);
                });
            }
            Err(e) => {
                println!("Error fetching bangs: {}", e);
                // If we have no bangs at all, use fallbacks
                if bangs.is_empty() {
                    bangs = FALLBACK_BANGS.clone();
                }
            }
        }
    }

    // Load and merge user custom bangs
    let user_bangs = load_user_bangs(app_handle);
    for (key, bang) in user_bangs {
        bangs.insert(key, bang);
    }

    bangs
}

// Get bang URL with query substitution
pub fn get_bang_url(bangs: &HashMap<String, Bang>, bang_id: &str, query: &str) -> Option<String> {
    bangs.get(bang_id).map(|bang| {
        let encoded_query = urlencoding::encode(query);
        // Handle different placeholder formats
        let url = bang
            .search_url
            .replace("{{{s}}}", &encoded_query)
            .replace("{{{qe}}}", &encoded_query)
            .replace("{{qe}}", &encoded_query)
            .replace("{{q}}", &query);

        url
    })
}

// Add or update a custom bang
pub fn add_custom_bang(
    app_handle: &AppHandle,
    all_bangs: &mut HashMap<String, Bang>,
    bang: Bang,
) -> Result<(), String> {
    // Load existing user bangs
    let mut user_bangs = load_user_bangs(app_handle);

    // Add or update the bang
    let mut custom_bang = bang;
    custom_bang.is_custom = true;

    user_bangs.insert(custom_bang.id.clone(), custom_bang.clone());
    all_bangs.insert(custom_bang.id.clone(), custom_bang);

    // Save user bangs
    save_user_bangs(app_handle, &user_bangs)
}

// Delete a custom bang
pub fn delete_custom_bang(
    app_handle: &AppHandle,
    all_bangs: &mut HashMap<String, Bang>,
    bang_id: &str,
) -> Result<(), String> {
    // Load existing user bangs
    let mut user_bangs = load_user_bangs(app_handle);

    // Remove the bang if it exists
    user_bangs.remove(bang_id);

    // If it was a custom bang, remove it from all_bangs
    // If it was a default bang, we need to reload it from cache
    if let Some(bang) = all_bangs.get(bang_id) {
        if bang.is_custom {
            all_bangs.remove(bang_id);
        } else if let Some(cache) = load_cache(app_handle) {
            if let Some(default_bang) = cache.bangs.get(bang_id) {
                all_bangs.insert(bang_id.to_string(), default_bang.clone());
            }
        }
    }

    // Save user bangs
    save_user_bangs(app_handle, &user_bangs)
}

// Add this function to get all bangs for the UI
pub fn get_all_bangs(bangs: &HashMap<String, Bang>) -> Vec<(String, String)> {
    bangs
        .iter()
        .map(|(id, bang)| (id.clone(), bang.name.clone()))
        .collect()
}
