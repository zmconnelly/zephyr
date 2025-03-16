use tauri::Manager;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    App,
};

use crate::{bangs, logger, search, updater};

pub fn initialize_tray(app: &App) {
    let version = app.package_info().version.to_string();

    let version_item = MenuItem::with_id(
        &app.handle().clone(),
        "version",
        format!("Version: {}", version),
        true,
        None::<&str>,
    )
    .unwrap();

    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();

    let open_logs_item = MenuItem::with_id(
        &app.handle().clone(),
        "open_logs",
        "Open Logs Folder",
        true,
        None::<&str>,
    )
    .unwrap();

    let refresh_bangs_item = MenuItem::with_id(
        &app.handle().clone(),
        "refresh_bangs",
        "Refresh Bangs",
        true,
        None::<&str>,
    )
    .unwrap();

    let menu = Menu::with_items(
        app,
        &[
            &version_item,
            &open_logs_item,
            &refresh_bangs_item,
            &quit_item,
        ],
    )
    .unwrap();

    TrayIconBuilder::new()
        .menu(&menu)
        .menu_on_left_click(true)
        .icon(app.default_window_icon().unwrap().clone())
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                logger::info("Quitting application...");
                app.exit(0);
            }
            "version" => {
                logger::info(&format!("Version: {}", env!("CARGO_PKG_VERSION")));
                if updater::is_dev_mode() {
                    logger::info("Skipping update check in development mode");
                } else {
                    let app_handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = updater::check_for_updates(app_handle).await {
                            logger::error(&format!("Update check failed: {}", e));
                        }
                    });
                }
            }
            "open_logs" => {
                logger::info("Opening logs folder...");
                if let Err(e) = logger::open_log_directory(app.clone()) {
                    logger::error(&format!("Failed to open logs folder: {}", e));
                }
            }
            "refresh_bangs" => {
                logger::info("Refreshing bangs cache...");
                let app_handle_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    match bangs::refresh_bangs(&app_handle_clone).await {
                        Ok(fresh_bangs) => {
                            if let Some(state) = app_handle_clone.try_state::<search::BangState>() {
                                let mut bangs_lock = state.bangs.lock().unwrap();
                                *bangs_lock = fresh_bangs;
                            }
                        }
                        Err(e) => {
                            logger::error(&format!("Failed to refresh bangs: {}", e));
                        }
                    }
                });
            }
            _ => {
                logger::debug(&format!("Menu item {:?} not handled", event.id));
            }
        })
        .build(app)
        .unwrap();
}
