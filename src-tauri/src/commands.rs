use std::path::PathBuf;
use tauri::{AppHandle, Listener, Manager, State, WebviewWindow, Window};
use tokio::sync::oneshot;
use crate::{config::Config, data_handler::DataHandler, db::DBHandler, model::{Entry, Subject}, state::SKState};

#[tauri::command]
pub async fn main_window_created(app: AppHandle, state: State<'_,SKState>, window: Window) -> Result<bool, String> {
    window.show().map_err(|e| e.to_string())?;

    let home_dir = {
        let state = state.lock().map_err(|e| e.to_string())?;
        println!("state: {:?}", state);

        if state.data.is_some() {
            return Ok(true);
        }

        match &state.c {
            Some(c) => Some(c.home_dir.clone()),
            None => None,
        }
    };

    let home_dir = match home_dir {
        Some(home_dir) => home_dir,
        None => {
            let config_path = Config::get_config_file_path(app.path()).map_err(|e| e.to_string())?;
            let c = match config_path.exists() { 
                true => Config::load(&config_path).map_err(|e| e.to_string())?,
                false => return Ok(false),
            };

            let home_dir = c.home_dir.clone();

            {
                let mut state = state.lock().map_err(|e| e.to_string())?;
                state.c = Some(c);
            };

            home_dir
        },
    };

    let mut db_dir = PathBuf::from(&home_dir);
    db_dir.push(".sk");

    println!("{db_dir:?}");

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
        let str = serde_json::from_str::<String>(e.payload()).unwrap();
        let _ = s.send(str);
    });

    let home_dir = r.await.map_err(|e| e.to_string())?;
    let c = Config::new(home_dir.clone(), app.path()).map_err(|e| e.to_string())?;

    let mut db_dir = PathBuf::from(&home_dir);
    db_dir.push(".sk");

    let dbh = match DBHandler::exists(&db_dir) {
        true => DBHandler::load(&db_dir).await,
        false => DBHandler::new(&db_dir).await,
    }.unwrap();

    let dh = DataHandler::new(dbh, &PathBuf::from(&home_dir)).await.unwrap();

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

//#[tauri::command]
//async fn init_main_window(app: AppHandle, home_dir: String) -> core::result::Result<(), String> {
//    let welcome = match app.get_webview_window("welcome") {
//        Some(welcome) => welcome,
//        None => return Err("Welcome window does not exist".into()),
//    };
//
//    let mut db_dir = PathBuf::from(&home_dir);
//    db_dir.push(".sk");
//
//    //let dbh = DBHandler::new(&db_dir).await.map_err(|e| e.to_string())?;
//    let dbh = match DBHandler::new(&db_dir).await {
//        Ok(dbh) => dbh,
//        Err(e) => return Err(e.to_string()),
//    };
//    let data = DataHandler::init(dbh, &PathBuf::from(&home_dir)).await;
//
//    let path_resolver = app.path();
//    let conf = Config::new(home_dir, &path_resolver).map_err(|e| e.to_string())?;
//
//    let state = app.state::<SKState>();
//    let mut state = state.lock().map_err(|e| e.to_string())?;
//    state.c = Some(conf);
//    state.data = Some(data);
//
//    show_main_window(&app, true).map_err(|e| e.to_string())?;
//    show_welcome_window(&app, false).map_err(|e| e.to_string())?;
//
//    Ok(())
//}

