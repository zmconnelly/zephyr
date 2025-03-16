use crate::bangs::models::{Bang, BangCache};
use crate::logger;
use chrono::Utc;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

// Fallback bangs to use if we can't fetch or load any
lazy_static! {
    pub(crate) static ref FALLBACK_BANGS: HashMap<String, Bang> = {
        let mut map = HashMap::new();

        // Add a few essential bangs as fallbacks
        map.insert(
            "g".to_string(),
            Bang {
                id: "g".to_string(),
                name: "Google".to_string(),
                search_url: "https://www.google.com/search?q={{{s}}}".to_string(),
                home_url: "https://www.google.com".to_string(),
                category: "Web - Search".to_string(),
                is_custom: false,
            },
        );

        map.insert(
            "w".to_string(),
            Bang {
                id: "w".to_string(),
                name: "Wikipedia".to_string(),
                search_url: "https://en.wikipedia.org/wiki/Special:Search?search={{{s}}}".to_string(),
                home_url: "https://en.wikipedia.org".to_string(),
                category: "Reference - Encyclopedia".to_string(),
                is_custom: false,
            },
        );

        map.insert(
            "yt".to_string(),
            Bang {
                id: "yt".to_string(),
                name: "YouTube".to_string(),
                search_url: "https://www.youtube.com/results?search_query={{{s}}}".to_string(),
                home_url: "https://www.youtube.com".to_string(),
                category: "Entertainment - Video".to_string(),
                is_custom: false,
            },
        );

        map.insert(
            "gh".to_string(),
            Bang {
                id: "gh".to_string(),
                name: "GitHub".to_string(),
                search_url: "https://github.com/search?q={{{s}}}".to_string(),
                home_url: "https://github.com".to_string(),
                category: "Tech - Programming".to_string(),
                is_custom: false,
            },
        );

        map.insert(
            "a".to_string(),
            Bang {
                id: "a".to_string(),
                name: "Amazon".to_string(),
                search_url: "https://www.amazon.com/s?k={{{s}}}".to_string(),
                home_url: "https://www.amazon.com".to_string(),
                category: "Shopping - General".to_string(),
                is_custom: false,
            },
        );

        map
    };
}

/// Get the path to the cache file
pub(crate) fn get_cache_path(_app_handle: &AppHandle) -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zephyr")
        .join("bangs_cache.json")
}

/// Get the path to the user settings file
pub(crate) fn get_settings_path(_app_handle: &AppHandle) -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zephyr")
        .join("user_bangs.json")
}

/// Load bangs from cache
pub(crate) fn load_cache(app_handle: &AppHandle) -> Option<BangCache> {
    let cache_path = get_cache_path(app_handle);

    if !cache_path.exists() {
        return None;
    }

    match fs::read_to_string(&cache_path) {
        Ok(json) => match serde_json::from_str::<BangCache>(&json) {
            Ok(cache) => Some(cache),
            Err(e) => {
                logger::error(&format!("Failed to parse bang cache: {}", e));
                None
            }
        },
        Err(e) => {
            logger::error(&format!("Failed to read bang cache: {}", e));
            None
        }
    }
}

/// Save bangs to cache
pub(crate) fn save_cache(
    app_handle: &AppHandle,
    bangs: &HashMap<String, Bang>,
) -> Result<(), String> {
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

/// Load user custom bangs
pub(crate) fn load_user_bangs(app_handle: &AppHandle) -> HashMap<String, Bang> {
    let settings_path = get_settings_path(app_handle);

    if !settings_path.exists() {
        return HashMap::new();
    }

    match fs::read_to_string(&settings_path) {
        Ok(json) => match serde_json::from_str::<HashMap<String, Bang>>(&json) {
            Ok(bangs) => bangs,
            Err(e) => {
                logger::error(&format!("Failed to parse user bangs: {}", e));
                HashMap::new()
            }
        },
        Err(e) => {
            logger::error(&format!("Failed to read user bangs: {}", e));
            HashMap::new()
        }
    }
}

/// Save user custom bangs
pub(crate) fn save_user_bangs(
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

/// Delete the cache file
pub(crate) fn delete_cache(app_handle: &AppHandle) -> Result<(), String> {
    let cache_path = get_cache_path(app_handle);

    if cache_path.exists() {
        fs::remove_file(&cache_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}
