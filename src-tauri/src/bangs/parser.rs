use crate::bangs::models::{Bang, DuckDuckGoBang};
use crate::logger;
use std::collections::{HashMap, HashSet};

/// Parse DuckDuckGo bang.js file into a HashMap of Bang objects
pub(crate) fn parse_duckduckgo_bangs(js_content: &str) -> Result<HashMap<String, Bang>, String> {
    let mut bangs = HashMap::new();

    // The bang.js file is a JavaScript array of objects
    // Parse the entire JSON array at once
    let js_content = js_content.trim();

    // Validate basic structure
    if !js_content.starts_with('[') || !js_content.ends_with(']') {
        return Err("Invalid bang.js format: not a JSON array".to_string());
    }

    // First try to parse with our structured approach
    let bang_array: Result<Vec<DuckDuckGoBang>, _> = serde_json::from_str(js_content);

    match bang_array {
        Ok(array) => {
            let total_bangs = array.len();
            logger::info(&format!("Successfully parsed {} bang entries", total_bangs));

            // Track seen triggers to handle duplicates
            let mut seen_triggers = HashSet::new();
            let mut skipped_count = 0;
            let mut invalid_count = 0;

            // Process each bang entry
            for ddg_bang in array {
                // Skip entries with missing required fields
                if !ddg_bang.is_valid() {
                    invalid_count += 1;
                    if invalid_count <= 5 {
                        // Limit logging to avoid spam
                        logger::warn("Skipping invalid bang entry: missing required fields");
                    }
                    continue;
                }

                // We can safely unwrap here because we checked is_valid()
                let trigger = ddg_bang.trigger.as_ref().unwrap();

                // Skip if we've already seen this trigger
                if seen_triggers.contains(trigger) {
                    skipped_count += 1;
                    if skipped_count <= 5 {
                        // Limit logging to avoid spam
                        logger::info(&format!("Skipping duplicate trigger: {}", trigger));
                    }
                    continue;
                }

                seen_triggers.insert(trigger.clone());

                if let Some((key, bang)) = ddg_bang.to_bang() {
                    bangs.insert(key, bang);
                }
            }

            logger::info(&format!(
                "Bang parsing summary: {} total, {} valid, {} duplicates skipped, {} invalid entries", 
                total_bangs, bangs.len(), skipped_count, invalid_count
            ));
        }
        Err(e) => {
            // If structured parsing fails, try a more lenient approach with Value
            logger::error(&format!(
                "Failed to parse bang.js with structured approach: {}",
                e
            ));
            logger::warn("Attempting fallback parsing with generic JSON Value...");

            // Try parsing as generic JSON Value array
            let fallback_array: Result<Vec<serde_json::Value>, _> =
                serde_json::from_str(js_content);

            match fallback_array {
                Ok(array) => {
                    let total_bangs = array.len();
                    logger::info(&format!(
                        "Successfully parsed {} bang entries with fallback method",
                        total_bangs
                    ));

                    // Track seen triggers to handle duplicates
                    let mut seen_triggers = HashSet::new();
                    let mut skipped_count = 0;
                    let mut invalid_count = 0;

                    // Process each bang entry
                    for bang_data in array {
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
                            // Skip if we've already seen this trigger
                            if seen_triggers.contains(trigger) {
                                skipped_count += 1;
                                if skipped_count <= 5 {
                                    // Limit logging to avoid spam
                                    logger::info(&format!(
                                        "Skipping duplicate trigger: {}",
                                        trigger
                                    ));
                                }
                                continue;
                            }

                            seen_triggers.insert(trigger.to_string());

                            let bang = Bang {
                                id: trigger.to_string(),
                                name: name.to_string(),
                                search_url: url.to_string(),
                                home_url: format!("https://{}", domain),
                                category: format!("{} - {}", category, subcategory),
                                is_custom: false,
                            };

                            bangs.insert(trigger.to_string(), bang);
                        } else {
                            invalid_count += 1;
                            if invalid_count <= 5 {
                                // Limit logging to avoid spam
                                logger::warn(
                                    "Skipping invalid bang entry: missing required fields",
                                );
                            }
                        }
                    }

                    logger::info(&format!(
                        "Bang fallback parsing summary: {} total, {} valid, {} duplicates skipped, {} invalid entries", 
                        total_bangs, bangs.len(), skipped_count, invalid_count
                    ));
                }
                Err(e) => {
                    return Err(format!("All parsing methods failed. JSON error: {}. Please check if DuckDuckGo has changed their bang.js format.", e));
                }
            }
        }
    }

    if bangs.is_empty() {
        return Err("No valid bangs found in the response".to_string());
    }

    Ok(bangs)
}

/// Fetch DuckDuckGo bangs from the official source
pub(crate) async fn fetch_duckduckgo_bangs() -> Result<HashMap<String, Bang>, String> {
    logger::info("Fetching DuckDuckGo bangs...");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get("https://duckduckgo.com/bang.js")
        .header("User-Agent", "Zephyr/1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch bangs: {}", e))?;

    let status = response.status();
    if status.is_success() {
        let js_content = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        logger::info(&format!(
            "Successfully fetched bang.js ({} bytes)",
            js_content.len()
        ));

        // Try to parse the bangs
        match parse_duckduckgo_bangs(&js_content) {
            Ok(bangs) => {
                logger::info(&format!(
                    "Successfully parsed {} bangs from DuckDuckGo",
                    bangs.len()
                ));
                Ok(bangs)
            }
            Err(e) => {
                logger::error(&format!("Error parsing bangs: {}", e));

                // If parsing fails but we have a valid response, save it for debugging
                if js_content.len() > 0 {
                    let debug_path = dirs::cache_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join("zephyr")
                        .join("debug_bang.js");

                    if let Some(parent) = debug_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }

                    if let Err(write_err) = std::fs::write(&debug_path, &js_content) {
                        logger::error(&format!("Failed to save debug bang.js: {}", write_err));
                    } else {
                        logger::info(&format!(
                            "Saved debug bang.js to {:?} for troubleshooting",
                            debug_path
                        ));
                    }
                }

                Err(format!("Error fetching bangs: {}", e))
            }
        }
    } else {
        let error_msg = format!("Failed to fetch bangs: HTTP {}", status);
        logger::error(&error_msg);
        Err(error_msg)
    }
}
