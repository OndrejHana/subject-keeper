// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use tauri::{DragDropEvent, Emitter, Manager, WindowEvent};

const BASE_DIR: &str = "sylabus";

#[tauri::command]
fn open_file(path: String) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    let opener = "xdg-open";

    #[cfg(target_os = "macos")]
    let opener = "open";

    #[cfg(target_os = "windows")]
    let opener = "cmd /c start";

    std::process::Command::new(opener)
        .arg(path)
        .spawn()
        .map_err(|err| format!("Failed to open file: {}", err))?;

    Ok(())
}

#[tauri::command]
fn open_folder(subject: String) {
    let subject_dir = home::home_dir().unwrap().join(BASE_DIR).join(&subject);

    #[cfg(target_os = "linux")]
    let opener = "xdg-open";

    #[cfg(target_os = "macos")]
    let opener = "open";

    #[cfg(target_os = "windows")]
    let opener = "explorer";

    std::process::Command::new(opener)
        .arg(subject_dir)
        .spawn()
        .unwrap();
}

#[tauri::command]
fn add_subject(name: String) {
    let home_dir = home::home_dir().unwrap().join(BASE_DIR);

    let _ = std::fs::create_dir_all(home_dir.join(name));
}

#[tauri::command]
fn get_subjects() -> Vec<String> {
    let home_dir = home::home_dir().unwrap().join(BASE_DIR);

    home_dir
        .read_dir()
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().into_string().unwrap())
        .filter(|entry| !entry.starts_with("."))
        .collect()
}

#[tauri::command]
fn get_subject(subject: String) -> Vec<(String, PathBuf)> {
    let home_dir = home::home_dir().unwrap().join(BASE_DIR);

    let subject_dir = home_dir.join(subject);

    subject_dir
        .read_dir()
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            let file_name = entry.file_name().into_string().unwrap();
            let path = entry.path();
            (file_name, path)
        })
        .filter(|(_, path)| path.is_file())
        .filter(|(_, path)| !path.file_name().unwrap().to_str().unwrap().starts_with("."))
        .collect()
}

#[tauri::command]
fn add_file(subject: String, path: String) {
    let home_dir = home::home_dir().unwrap().join(BASE_DIR);

    let path = PathBuf::from(path);
    let file_name = path.file_name().unwrap().to_str().unwrap();

    std::fs::copy(&path, home_dir.join(subject).join(file_name)).unwrap();
}

fn main() {
    tauri::Builder::default()
        .setup(|_app| {
            let home_dir = home::home_dir().unwrap().join(BASE_DIR);

            if !home_dir.exists() {
                std::fs::create_dir_all(&home_dir).unwrap();
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::DragDrop(e) = event {
                match e {
                    DragDropEvent::Drop { paths, position } => {
                        let path = paths.first().unwrap();
                        window.emit("fileDropped", path).unwrap();
                    }
                    _ => {}
                }
            }
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            add_subject,
            get_subjects,
            get_subject,
            add_file,
            open_file,
            open_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
