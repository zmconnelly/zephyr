use crate::bangs::models::{Bang, BangCache};
use crate::logger;
use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

pub(crate) fn get_cache_path(_app_handle: &AppHandle) -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zephyr")
        .join("bangs_cache.json")
}

pub(crate) fn get_settings_path(_app_handle: &AppHandle) -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zephyr")
        .join("user_bangs.json")
}

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

pub(crate) fn save_cache(
    app_handle: &AppHandle,
    bangs: &HashMap<String, Bang>,
) -> Result<(), String> {
    let cache_path = get_cache_path(app_handle);

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

pub(crate) fn save_user_bangs(
    app_handle: &AppHandle,
    bangs: &HashMap<String, Bang>,
) -> Result<(), String> {
    let settings_path = get_settings_path(app_handle);

    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let json = serde_json::to_string_pretty(&bangs).map_err(|e| e.to_string())?;
    fs::write(&settings_path, json).map_err(|e| e.to_string())
}

pub(crate) fn delete_cache(app_handle: &AppHandle) -> Result<(), String> {
    let cache_path = get_cache_path(app_handle);

    if cache_path.exists() {
        fs::remove_file(&cache_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}
