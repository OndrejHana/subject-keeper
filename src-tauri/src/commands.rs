#[tauri::command]
fn get_all_subjects(state: tauri::State<SKState>) -> core::result::Result<Vec<Subject>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    match &state.data {
        Some(dh) => Ok(dh.get_all_subjects()),
        None => Err("state not found".into()),
    }
}

#[tauri::command]
fn get_all_entries(state: tauri::State<SKState>) -> core::result::Result<Vec<Entry>, String> {
    let state = state.lock().unwrap();
    match &state.data {
        Some(dh) => Ok(dh.get_all_entries()),
        None => Err("state not found".into()),
    }
}

#[tauri::command]
async fn init_main_window(app: AppHandle, home_dir: String) -> core::result::Result<(), String> {
    let welcome = match app.get_webview_window("welcome") {
        Some(welcome) => welcome,
        None => return Err("Welcome window does not exist".into()),
    };

    let mut db_dir = PathBuf::from(&home_dir);
    db_dir.push(".sk");

    //let dbh = DBHandler::new(&db_dir).await.map_err(|e| e.to_string())?;
    let dbh = match DBHandler::new(&db_dir).await {
        Ok(dbh) => dbh,
        Err(e) => return Err(e.to_string()),
    };
    let data = DataHandler::init(dbh, &PathBuf::from(&home_dir)).await;

    let path_resolver = app.path();
    let conf = Config::new(home_dir, &path_resolver).map_err(|e| e.to_string())?;

    let state = app.state::<SKState>();
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.c = Some(conf);
    state.data = Some(data);

    show_main_window(&app, true).map_err(|e| e.to_string())?;
    show_welcome_window(&app, false).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn setup_main_window(app: AppHandle) -> core::result::Result<bool, String> {
    let home_dir = {
        let state = app.state::<SKState>();
        let state = state.lock().unwrap();

        if state.data.is_some() {
            return Ok(true);
        }

        match state.c.borrow(){
            Some(c) => c.home_dir.clone(),
            None => {
                show_welcome_window(app.app_handle(), true).map_err(|e| e.to_string())?;
                return Ok(false);
            }
        }
    };

    let mut db_dir = PathBuf::from(&home_dir);
    db_dir.push(".sk");

    let dbh = DBHandler::new(&db_dir).await.map_err(|e| e.to_string())?;
    let data = DataHandler::init(dbh, &PathBuf::from(&home_dir)).await;

    {
        let state = app.state::<SKState>();
        let mut state = state.lock().unwrap();
        state.data = Some(data);
    }

    Ok(true)
}
