use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

use crate::logger;

// Function to get the GitHub token from environment variable
fn get_bundled_token() -> &'static str {
    option_env!("GITHUB_TOKEN").unwrap_or("")
}

// Check for updates
#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> tauri_plugin_updater::Result<()> {
    logger::info("Checking for updates...");

    // Get the token (for debugging only)
    let token = get_bundled_token();
    if !token.is_empty() {
        logger::debug("GitHub token is set from environment variable");
        logger::debug(&format!("Token: {}", token));
    } else {
        logger::debug("No GitHub token found in environment variables");
    }

    // Use the updater directly - the token should be set at build time
    let updater = app.updater()?;

    // Log the endpoints from tauri.conf.json for debugging
    logger::debug("Using updater with GitHub token from environment variable");
    logger::debug("If updates fail, check your GitHub token and repository permissions");

    match updater.check().await {
        Ok(Some(update)) => {
            logger::info(&format!(
                "Update found: {} (current: {})",
                update.version,
                env!("CARGO_PKG_VERSION")
            ));

            match update.download_and_install(|_, _| {}, || {}).await {
                Ok(_) => logger::info("Update installed successfully"),
                Err(e) => logger::error(&format!("Failed to install update: {}", e)),
            }
        }
        Ok(None) => {
            logger::info("No updates available");
        }
        Err(e) => {
            logger::error(&format!("Error checking for updates: {}", e));
            return Err(e);
        }
    }

    Ok(())
}

// Store GitHub token (kept for compatibility)
#[tauri::command]
pub fn set_github_token(_app: AppHandle, _token: String) -> Result<(), String> {
    logger::warn("Setting GitHub token at runtime is not supported in this version");
    logger::info("Please set the GITHUB_TOKEN environment variable at build time instead");
    Ok(())
}

// Get stored GitHub token (kept for compatibility)
#[tauri::command]
pub fn get_github_token(_app: AppHandle) -> Result<String, String> {
    Ok(get_bundled_token().to_string())
}

// Update GitHub URL (kept for compatibility)
#[tauri::command]
pub fn update_github_url(_app: AppHandle, _token: String) -> Result<(), String> {
    logger::warn("Updating GitHub URL at runtime is not supported in this version");
    logger::info("Please set the GITHUB_TOKEN environment variable at build time instead");
    Ok(())
}

#[tauri::command]
pub async fn test_github_token() -> Result<String, String> {
    let token = get_bundled_token();
    if token.is_empty() {
        return Err("No GitHub token found".to_string());
    }

    // Create a client with the token
    let client = reqwest::Client::new();

    // Test URL - this should be accessible with a valid token with repo scope
    let test_url = "https://api.github.com/repos/zmconnelly/zephyr/releases/latest";

    // Make the request
    let response = client
        .get(test_url)
        .header("Authorization", format!("token {}", token))
        .header("User-Agent", "Zephyr-App")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    // Check the status
    let status = response.status();
    if status.is_success() {
        Ok(format!("Token is valid! Status: {}", status))
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!(
            "Token validation failed. Status: {}, Error: {}",
            status, error_text
        ))
    }
}
