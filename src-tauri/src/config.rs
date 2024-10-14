use crate::error::*;

use std::{fs::{remove_file, File}, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};
use tauri::{path::PathResolver, Runtime};

pub const DB_FILENAME: &str = "skdata.db";

const APP_CONFIG_FILE: &str = "sk.json";
const HOMEDIR_CONFIG_DIR: &str = ".sk";

const DEFAULT_WELCOME_WINDOW_SIZE: (f64, f64) = (400.0, 600.0);
const DEFAULT_MAIN_WINDOW_SIZE: (f64, f64) = (800.0, 600.0);
const DEFAULT_SIDEBAR_PERCENT: u64 = 15;
const DEFAULT_PREVIEW_PERCENT: u64 = 20;


#[derive(Serialize, Deserialize)]
pub struct Config {
    pub home_dir: String,
    pub db_dirname: String,
    pub width: f64,
    pub height: f64,
    pub sidebar_size: u64,
    pub entry_preview_size: u64,
}

impl Config {
    pub fn new<R>(home_dir: String, path_resolver: &PathResolver<R>) -> Result<Config>
    where
        R: Runtime,
    {
        let mut db_dir_path = PathBuf::from(&home_dir);
        db_dir_path.push(HOMEDIR_CONFIG_DIR);
        let db_dirname = db_dir_path.to_string_lossy().to_string();

        let conf = Config {
            home_dir,
            db_dirname,
            width: DEFAULT_MAIN_WINDOW_SIZE.0,
            height: DEFAULT_MAIN_WINDOW_SIZE.1,
            sidebar_size: DEFAULT_SIDEBAR_PERCENT,
            entry_preview_size: DEFAULT_PREVIEW_PERCENT,
        };
        let config_file_path = Config::get_config_file_path(path_resolver)?;
        conf.create_config_file(&config_file_path)?;
        conf.store(&config_file_path)?;

        Ok(conf)
    }
    pub fn get_config_file_path<R>(path_resolver: &PathResolver<R>) -> Result<PathBuf>
    where
        R: Runtime,
    {
        let path = path_resolver.app_config_dir()?.join(APP_CONFIG_FILE);
        Ok(path)
    }
    pub fn load(config_path: &Path) -> Result<Self> {
        let config_raw = std::fs::read(config_path)?;
        let config: Self = serde_json::from_slice(&config_raw)?;
        Ok(config)
    }
    pub fn store(&self, path: &Path) -> Result<()> {
        let config_raw = serde_json::to_vec_pretty(self)?;
        let parent = path.parent().unwrap();
        std::fs::create_dir_all(parent)?;
        std::fs::write(path, config_raw)?;
        Ok(())
    }
    pub fn create_config_file(&self, path: &Path) -> Result<File> {
        let config_dir = path.parent().ok_or("path is in root folder")?;
        std::fs::create_dir_all(config_dir)?;
        Ok(std::fs::File::create(&path)?)
    }
    pub fn get_db_file_path(&self) -> PathBuf {
        let mut path = PathBuf::from(&self.db_dirname);
        path.push(DB_FILENAME);
        path
    }
    pub fn set_home_dir(&mut self, home_dir: String) {
        self.home_dir = home_dir;
    }

    pub fn delete_config_file(path: &Path) -> Result<()> {
        remove_file(path)?;
        Ok(())
    }
}


