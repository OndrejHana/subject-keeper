use std::sync::Mutex;
use state::{SKState, SKStateInner};
use crate::commands::*;

mod error;
mod config;
mod model;
mod db;
mod data_handler;
mod state;
mod startup;
mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![main_window_created, select_home_dir, get_all_subjects, get_all_entries])
        .manage::<SKState>(Mutex::new(SKStateInner {
            data: None,
            c: None,
        }))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
