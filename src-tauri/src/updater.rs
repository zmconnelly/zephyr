use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

use crate::logger;

pub fn is_dev_mode() -> bool {
    cfg!(dev)
}

pub async fn check_for_updates(app: AppHandle) -> tauri_plugin_updater::Result<()> {
    logger::info("Checking for updates...");

    match app.updater()?.check().await {
        Ok(Some(update)) => {
            logger::info(&format!(
                "Update found: {} (current: {})",
                update.version,
                app.package_info().version.to_string()
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
