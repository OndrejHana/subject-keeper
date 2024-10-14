use std::path::PathBuf;
use tauri::{AppHandle, Listener, Manager, State};
use tokio::sync::oneshot;
use crate::{config::Config, data_handler::DataHandler, db::DBHandler, model::{Entry, Subject}, state::SKState};

#[tauri::command]
pub async fn main_window_created(state: State<'_,SKState>) -> Result<bool, String> {
    let home_dir = {
        let state = state.lock().map_err(|e| e.to_string())?;

        if state.data.is_some() {
            return Ok(true);
        }

        match &state.c {
            Some(c) => c.home_dir.clone(),
            None => return Ok(false),
        }
    };

    let mut db_dir = PathBuf::from(&home_dir);
    db_dir.push(".sk");

    let dbh = match DBHandler::exists(&db_dir) {
        true => DBHandler::load(&db_dir).await,
        false => DBHandler::new(&db_dir).await,
    }.map_err(|e| e.to_string())?;

    let dh = DataHandler::new(dbh, &PathBuf::from(&home_dir)).await.map_err(|e| e.to_string())?;

    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.data = Some(dh);
    Ok(true)
}

#[tauri::command]
pub async fn select_home_dir(app: AppHandle, state: State<'_, SKState>) -> Result<(), String> {
    let main = app.get_webview_window("main").ok_or("could not get main window")?;
    let welcome = app.get_webview_window("welcome").ok_or("could not get welcome window")?;

    main.hide().map_err(|e| e.to_string())?;
    welcome.show().map_err(|e| e.to_string())?;

    let (s, r) = oneshot::channel::<String>();
    app.once("home-dir-selected", |e| {
        let home_dir = e.payload();
        let _ = s.send(home_dir.to_string());
    });

    let home_dir = r.await.map_err(|e| e.to_string())?;

    let c = Config::new(home_dir.clone(), app.path()).map_err(|e| e.to_string())?;

    let mut db_dir = PathBuf::from(&home_dir);
    db_dir.push(".sk");

    let dbh = match DBHandler::exists(&db_dir) {
        true => DBHandler::load(&db_dir).await,
        false => DBHandler::new(&db_dir).await,
    }.map_err(|e| e.to_string())?;

    let dh = DataHandler::new(dbh, &PathBuf::from(&home_dir)).await.map_err(|e| e.to_string())?;

    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.c = Some(c);
    state.data = Some(dh);

    main.show().map_err(|e| e.to_string())?;
    welcome.hide().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_all_subjects(state: tauri::State<SKState>) -> core::result::Result<Vec<Subject>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    println!("{:?}", state.data);
    match &state.data {
        Some(dh) => Ok(dh.get_all_subjects()),
        None => Err("state not found".into()),
    }
}

#[tauri::command]
pub fn get_all_entries(state: tauri::State<SKState>) -> core::result::Result<Vec<Entry>, String> {
    let state = state.lock().unwrap();
    match &state.data {
        Some(dh) => Ok(dh.get_all_entries()),
        None => Err("state not found".into()),
    }
}
