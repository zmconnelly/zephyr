// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::panic;

fn main() {
    panic::set_hook(Box::new(|panic_info| {
        println!("Panic occurred: {:?}", panic_info);
    }));

    if let Err(e) = std::panic::catch_unwind(|| zephyr_lib::run()) {
        println!("Application crashed: {:?}", e);
    }
}
