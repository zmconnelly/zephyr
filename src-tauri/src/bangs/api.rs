use crate::bangs::models::Bang;
use crate::bangs::parser::fetch_duckduckgo_bangs;
use crate::bangs::storage::{
    delete_cache, load_cache, load_user_bangs, save_cache, FALLBACK_BANGS,
};
use crate::logger;
use chrono::{Duration, Utc};
use std::collections::HashMap;
use tauri::AppHandle;

/// Load all bangs (DuckDuckGo + user custom)
pub async fn load_all_bangs(app_handle: &AppHandle) -> HashMap<String, Bang> {
    // First, try to load from cache
    let mut should_update = false;
    let mut bangs = if let Some(cache) = load_cache(app_handle) {
        // Check if cache is older than 7 days
        let now = Utc::now();
        if now - cache.last_updated > Duration::days(7) {
            should_update = true;
            logger::info("Bang cache is older than 7 days, will attempt to update");
        } else {
            logger::info(&format!(
                "Using bang cache with {} entries (last updated: {})",
                cache.bangs.len(),
                cache.last_updated
            ));
        }
        cache.bangs
    } else {
        // No cache, need to fetch
        should_update = true;
        logger::info("No bang cache found, will fetch from DuckDuckGo");
        HashMap::new()
    };

    // If cache is empty or old, try to update it
    if should_update {
        match fetch_duckduckgo_bangs().await {
            Ok(fetched_bangs) => {
                logger::info(&format!(
                    "Successfully fetched {} bangs from DuckDuckGo",
                    fetched_bangs.len()
                ));

                // Only replace existing bangs if we got a reasonable number of new ones
                // This prevents replacing a good cache with a bad response
                if fetched_bangs.len() > 100 || bangs.is_empty() {
                    bangs = fetched_bangs;

                    // Save to cache in the background
                    let app_handle_clone = app_handle.clone();
                    let bangs_clone = bangs.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = save_cache(&app_handle_clone, &bangs_clone) {
                            logger::error(&format!("Failed to save bang cache: {}", e));
                        } else {
                            logger::info(&format!(
                                "Successfully saved {} bangs to cache",
                                bangs_clone.len()
                            ));
                        }
                    });
                } else {
                    logger::warn(&format!(
                        "Fetched only {} bangs, which seems suspiciously low. Keeping existing {} bangs from cache.",
                        fetched_bangs.len(), bangs.len()
                    ));
                }
            }
            Err(e) => {
                logger::error(&format!("Error fetching bangs: {}", e));

                // If we have no bangs at all, use fallbacks
                if bangs.is_empty() {
                    logger::warn("Using fallback bangs since cache is empty and fetch failed");
                    bangs = FALLBACK_BANGS.clone();
                } else {
                    logger::info(&format!(
                        "Continuing to use {} bangs from cache despite fetch error",
                        bangs.len()
                    ));
                }
            }
        }
    }

    // Load and merge user custom bangs
    let user_bangs = load_user_bangs(app_handle);
    logger::info(&format!("Loaded {} custom user bangs", user_bangs.len()));

    for (key, bang) in user_bangs {
        bangs.insert(key, bang);
    }

    logger::info(&format!("Total bangs available: {}", bangs.len()));
    bangs
}

/// Get bang URL with query substitution
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

/// Add or update a custom bang
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

/// Delete a custom bang
pub fn delete_custom_bang(
    app_handle: &AppHandle,
    all_bangs: &mut HashMap<String, Bang>,
    bang_id: &str,
) -> Result<(), String> {
    // Load existing user bangs
    let mut user_bangs = load_user_bangs(app_handle);

    // Check if the bang exists and is custom
    if let Some(bang) = all_bangs.get(bang_id) {
        if !bang.is_custom {
            return Err(format!("Cannot delete built-in bang: {}", bang_id));
        }
    } else {
        return Err(format!("Bang not found: {}", bang_id));
    }

    // Remove the bang
    user_bangs.remove(bang_id);
    all_bangs.remove(bang_id);

    // Save user bangs
    save_user_bangs(app_handle, &user_bangs)
}

/// Get all bangs as a list of (id, name) tuples
pub fn get_all_bangs(bangs: &HashMap<String, Bang>) -> Vec<(String, String)> {
    bangs
        .iter()
        .map(|(id, bang)| (id.clone(), bang.name.clone()))
        .collect()
}

/// Clear the bang cache
pub fn clear_cache(app_handle: &AppHandle) -> Result<(), String> {
    delete_cache(app_handle)
}

/// Refresh bangs from DuckDuckGo
pub async fn refresh_bangs(app_handle: &AppHandle) -> Result<HashMap<String, Bang>, String> {
    // First clear the cache
    clear_cache(app_handle)?;

    // Then fetch new bangs
    match fetch_duckduckgo_bangs().await {
        Ok(fetched_bangs) => {
            // Save to cache
            let bangs_clone = fetched_bangs.clone();
            if let Err(e) = save_cache(app_handle, &bangs_clone) {
                logger::error(&format!("Failed to save refreshed bangs to cache: {}", e));
            }

            // Load and merge user custom bangs
            let mut all_bangs = fetched_bangs;
            let user_bangs = load_user_bangs(app_handle);

            for (key, bang) in user_bangs {
                all_bangs.insert(key, bang);
            }

            Ok(all_bangs)
        }
        Err(e) => Err(e),
    }
}

/// Save user bangs (public wrapper for the storage function)
pub fn save_user_bangs(
    app_handle: &AppHandle,
    bangs: &HashMap<String, Bang>,
) -> Result<(), String> {
    crate::bangs::storage::save_user_bangs(app_handle, bangs)
}
