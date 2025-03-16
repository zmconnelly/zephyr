use crate::logger;
use serde::{Deserialize, Serialize};
use std::env;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Debug, Serialize, Deserialize)]
pub struct StartupStatus {
    pub enabled: bool,
}

#[tauri::command]
pub fn get_startup_status() -> Result<StartupStatus, String> {
    logger::debug("Checking startup status");

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = match hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run") {
        Ok(key) => key,
        Err(_) => return Ok(StartupStatus { enabled: false }),
    };

    let is_enabled = run_key.get_value::<String, _>("Zephyr").is_ok();

    logger::debug(&format!("Startup status: {}", is_enabled));
    Ok(StartupStatus {
        enabled: is_enabled,
    })
}

#[tauri::command]
pub fn toggle_run_at_startup(enable: bool) -> Result<StartupStatus, String> {
    logger::info(&format!("Toggling run at startup: {}", enable));

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let run_key_result = hkcu.open_subkey_with_flags(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        KEY_WRITE,
    );

    let run_key = match run_key_result {
        Ok(key) => key,
        Err(_) => {
            let (key, _) = hkcu
                .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
                .map_err(|e| e.to_string())?;
            key
        }
    };

    if enable {
        let exe_path = env::current_exe()
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .to_string();

        logger::debug(&format!("Adding to startup: {}", exe_path));
        run_key
            .set_value("Zephyr", &exe_path)
            .map_err(|e| e.to_string())?;
    } else {
        logger::debug("Removing from startup");
        let _ = run_key.delete_value("Zephyr");
    }

    Ok(StartupStatus { enabled: enable })
}
