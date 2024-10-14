mod error;
mod config;
mod model;
mod db;
mod data_handler;
mod state;
mod startup;
mod commands;

//fn show_welcome_window(app: &AppHandle, visible: bool) -> Result<()> {
//    let w = app
//        .get_webview_window("welcome")
//        .ok_or("could not get a window")?;
//
//    match visible {
//        true => w.show(),
//        false => w.hide() ,
//    }?;
//
//    Ok(())
//}
//
//fn show_main_window(app: &AppHandle, visible: bool) -> Result<()> {
//    let w = app.get_webview_window("main").ok_or("could not get a window")?;
//
//    match visible {
//        true => w.show(),
//        false => w.hide(),
//    }?;
//
//    Ok(())
//}




#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![init_main_window, get_all_entries, get_all_subjects, setup_main_window])
        .manage::<SKState>(Mutex::new(SKStateInner {
            data: None,
            c: None,
        }))
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    println!("{:?}", setup(app.handle()).await);

    app.run(|_app, _e| {});
}
