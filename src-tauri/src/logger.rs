#![allow(unused)]

use flexi_logger::{DeferredNow, Duplicate, FileSpec, Logger, WriteMode};
use log::Record;
use std::{fs, io::Write, path::Path, process::Command};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

pub fn init() {
    let file_format = |write: &mut dyn Write, now: &mut DeferredNow, record: &Record| {
        write!(
            write,
            "{} [{}] - {} - {}",
            now.now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.target(),
            record.args()
        )
    };

    let terminal_format = |write: &mut dyn Write, now: &mut DeferredNow, record: &Record| {
        let level = record.level();
        let color_code = match level {
            log::Level::Error => "\x1b[31m", // Red
            log::Level::Warn => "\x1b[33m",  // Yellow
            log::Level::Info => "\x1b[32m",  // Green
            log::Level::Debug => "\x1b[34m", // Blue
            log::Level::Trace => "\x1b[35m", // Magenta
        };

        write!(
            write,
            "{}{} [{}] - {} - {}\x1b[0m",
            color_code,
            now.now().format("%Y-%m-%d %H:%M:%S"),
            level,
            record.target(),
            record.args()
        )
    };

    let log_file_spec = FileSpec::default()
        .directory("logs")
        .basename("zephyr")
        .suffix("log");

    Logger::try_with_str("info")
        .unwrap()
        .format_for_files(file_format)
        .format_for_stdout(terminal_format)
        .log_to_file(log_file_spec)
        .write_mode(WriteMode::BufferAndFlush)
        .duplicate_to_stdout(Duplicate::All)
        .start()
        .unwrap();

    delete_old_log_files(3);
}

fn delete_old_log_files(keep: usize) {
    let logs_dir = Path::new("logs");
    if !logs_dir.exists() {
        return;
    }

    let mut log_files: Vec<_> = fs::read_dir(logs_dir)
        .unwrap_or_else(|e| {
            log::error!("Failed to read logs directory: {}", e);
            return fs::read_dir(logs_dir).unwrap();
        })
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("log") {
                Some((entry.metadata().ok()?.modified().ok()?, path))
            } else {
                None
            }
        })
        .collect();

    if log_files.len() <= keep {
        return;
    }

    log_files.sort_by(|a, b| b.0.cmp(&a.0));

    for (_, path) in log_files.into_iter().skip(keep) {
        log::debug!("Deleting old log file: {}", path.display());
        let _ = fs::remove_file(&path).map_err(|e| {
            log::error!("Failed to delete old log file {}: {}", path.display(), e);
        });
    }
}

pub fn open_log_directory(app_handle: AppHandle) -> Result<(), String> {
    let logs_dir = Path::new("logs");

    if !logs_dir.exists() {
        fs::create_dir_all(logs_dir)
            .map_err(|e| format!("Failed to create logs directory: {}", e))?;
    }

    let path = logs_dir
        .canonicalize()
        .map_err(|e| format!("Failed to get absolute path to logs directory: {}", e))?
        .to_string_lossy()
        .to_string();

    log::info!("Opening logs folder: {}", path);

    app_handle
        .opener()
        .open_path(path, None::<&str>)
        .map_err(|e| format!("Failed to open log directory: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn log(message: String, level: &str) {
    match level {
        "debug" => log::debug!("UI: {}", message),
        "info" => log::info!("UI: {}", message),
        "warn" => log::warn!("UI: {}", message),
        "error" => log::error!("UI: {}", message),
        _ => log::info!("UI: {}", message),
    }
}

pub fn error(message: &str) {
    log::error!("{}", message);
}

pub fn warn(message: &str) {
    log::warn!("{}", message);
}

pub fn info(message: &str) {
    log::info!("{}", message);
}

pub fn debug(message: &str) {
    log::debug!("{}", message);
}

pub fn trace(message: &str) {
    log::trace!("{}", message);
}
