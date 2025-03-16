use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

use crate::logger;

pub fn is_dev_mode() -> bool {
    cfg!(dev)
}

#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> tauri_plugin_updater::Result<()> {
    logger::info("Checking for updates...");

    // Get the token (for debugging only)
    let token = option_env!("GITHUB_TOKEN").unwrap_or("");
    if !token.is_empty() {
        logger::debug("GitHub token is set from environment variable");
        logger::debug(&format!("Token: {}", token));
    } else {
        logger::debug("No GitHub token found in environment variables");
    }

    let updater = app.updater()?;

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
