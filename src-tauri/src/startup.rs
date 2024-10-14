use std::path::PathBuf;

use crate::{config::Config, data_handler::DataHandler, db::DBHandler, error::*, state::SKState};
use tauri::{AppHandle, Manager};

pub async fn setup(app: &AppHandle) -> Result<()> {
    println!("1");

    let app_config_file = Config::get_config_file_path(app.path())?;
    println!("2");
    let c = Config::load(&app_config_file)?;
    println!("3");
    let home_dir = PathBuf::from(&c.home_dir);

    let state = app.state::<SKState>();
    let mut state = state.lock().unwrap();

    state.c = Some(c);
    println!("4");

    let mut db_dir = home_dir.clone();
    db_dir.push(".sk");
    let dbh = DBHandler::new(&db_dir).await?;
    println!("5");
    let data = DataHandler::init(dbh, &home_dir).await;
    println!("6");

    state.data = Some(data);

    Ok(())
}
