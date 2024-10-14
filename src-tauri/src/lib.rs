use std::{
    borrow::Borrow, collections::HashMap, fs::{self, File, OpenOptions}, path::{Path, PathBuf}, sync::Mutex
};

use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, Executor, SqlitePool};
use tauri::{path::PathResolver, AppHandle, Manager, Runtime};
use uuid::Uuid;

const APP_CONFIG_FILE: &str = "sk.json";
const HOMEDIR_CONFIG_DIR: &str = ".sk";
const DB_FILENAME: &str = "skdata.db";

const DEFAULT_WELCOME_WINDOW_SIZE: (f64, f64) = (400.0, 600.0);
const DEFAULT_MAIN_WINDOW_SIZE: (f64, f64) = (800.0, 600.0);
const DEFAULT_SIDEBAR_PERCENT: u64 = 15;
const DEFAULT_PREVIEW_PERCENT: u64 = 20;

const DB_MIGRATION: &str = r#"
CREATE TABLE IF NOT EXISTS subjects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    icon TEXT
);

CREATE TABLE IF NOT EXISTS entries (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    parent TEXT NOT NULL,
    FOREIGN KEY (parent) REFERENCES subjects(id)
);

CREATE TABLE IF NOT EXISTS fields(
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    template JSON NOT NULL
);

CREATE TABLE IF NOT EXISTS assigned_fields(
    id TEXT PRIMARY KEY,
    entry TEXT NOT NULL,
    field TEXT NOT NULL,
    value JSON NOT NULL,
    FOREIGN KEY (entry) REFERENCES entries(id),
    FOREIGN KEY (field) REFERENCES fields(id)
);

CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    color VARCHAR(7) NOT NULL,
    icon TEXT
);

CREATE TABLE IF NOT EXISTS tag_aliases (
    id TEXT PRIMARY KEY,
    alias TEXT NOT NULL,
    tag TEXT NOT NULL,
    FOREIGN KEY (tag) REFERENCES tags(id)
);

CREATE TABLE IF NOT EXISTS tag_relations(
    id TEXT PRIMARY KEY,
    parent TEXT NOT NULL,
    child TEXT NOT NULL,
    FOREIGN KEY (parent) REFERENCES tags(id),
    FOREIGN KEY (child) REFERENCES tags(id)
);

CREATE TABLE IF NOT EXISTS assigned_tags(
    id TEXT PRIMARY KEY,
    entry TEXT NOT NULL,
    tag TEXT NOT NULL,
    FOREIGN KEY (entry) REFERENCES entries(id),
    FOREIGN KEY (tag) REFERENCES tags(id)
);
"#;

type Error = Box<dyn std::error::Error>;
type Result<T> = core::result::Result<T, Error>;

#[derive(Serialize, Deserialize)]
struct Config {
    home_dir: String,
    db_dirname: String,
    width: f64,
    height: f64,
    sidebar_size: u64,
    entry_preview_size: u64,
}

impl Config {
    fn new<R>(home_dir: String, path_resolver: &PathResolver<R>) -> Result<Config>
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
    pub fn load(path: &Path) -> Result<Self> {
        let config_raw = std::fs::read(path)?;
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
        fs::remove_file(path)?;
        Ok(())
    }
}


fn show_welcome_window(app: &AppHandle, visible: bool) -> Result<()> {
    let w = app
        .get_webview_window("welcome")
        .ok_or("could not get a window")?;

    match visible {
        true => w.show(),
        false => w.hide() ,
    }?;

    Ok(())
}

fn show_main_window(app: &AppHandle, visible: bool) -> Result<()> {
    let w = app.get_webview_window("main").ok_or("could not get a window")?;

    match visible {
        true => w.show(),
        false => w.hide(),
    }?;

    Ok(())
}


#[derive(sqlx::FromRow, Debug)]
struct DBEntry {
    id: String,
    name: String,
    path: String,
    parent: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Entry {
    id: uuid::Uuid,
    name: String,
    path: PathBuf,
    parent: uuid::Uuid,
    exists: bool,
}

impl Entry {
    fn from_db(s: &DBEntry, exists: bool) -> Self {
        let path = PathBuf::from(&s.path);
        Self {
            id: Uuid::parse_str(&s.id).unwrap(),
            name: s.name.clone(),
            path,
            parent: Uuid::parse_str(&s.parent).unwrap(),
            exists,
        }
    }
    fn new(path: PathBuf, parent_id: Uuid, exists: bool) -> Self {
        let id = Uuid::now_v7();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        Self {
            id,
            name,
            path,
            parent: parent_id,
            exists,
        }
    }
}

#[derive(sqlx::FromRow, Debug)]
struct DBSubject {
    id: String,
    name: String,
    path: String,
    icon: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Subject {
    id: Uuid,
    name: String,
    path: PathBuf,
    icon: Option<String>,
    exists: bool,
}

impl Subject {
    fn new(path: PathBuf, icon: Option<String>, exists: bool) -> Self {
        let id = Uuid::now_v7();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        Self {
            id,
            name,
            path,
            icon,
            exists,
        }
    }
    fn from_db(s: &DBSubject, exists: bool) -> Self {
        Self {
            id: Uuid::parse_str(&s.id).unwrap(),
            name: s.name.clone(),
            path: PathBuf::from(&s.path),
            icon: s.icon.clone(),
            exists,
        }
    }
}

#[derive(sqlx::FromRow, Debug)]
struct DBTagAlias {
    id: String,
    alias: String,
    tag: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TagAlias {
    id: String,
    alias: String,
    tag: String,
}

#[derive(sqlx::FromRow, Debug)]
struct DBTagRelation {
    id: String,
    parent: String,
    child: String,
}

#[derive(sqlx::FromRow, Debug)]
struct DBTag {
    id: String,
    name: String,
    color: String,
    icon: String,
}



#[derive(Debug, Serialize, Deserialize, Clone)]
struct Tag {
    id: uuid::Uuid,
    name: String,
    color: String,
    icon: String,
    aliases: Vec<String>,
    parent: Uuid,
    children: Vec<Uuid>,
}

struct DBHandler {
    db: SqlitePool,
}

impl DBHandler {
    async fn new(config_dir: &Path) -> Result<Self> {
        fs::create_dir_all(&config_dir)?;
        let mut db_path = PathBuf::from(config_dir);
        db_path.push(DB_FILENAME);
        OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&db_path)?;
        let db = SqlitePoolOptions::new()
            .connect(db_path.to_str().unwrap())
            .await?;
        db.execute(DB_MIGRATION).await.unwrap();
        Ok(DBHandler { db })
    }
    async fn open(path: &Path) -> Result<Self> {
        let db = SqlitePoolOptions::new()
            .connect(path.to_str().unwrap())
            .await?;
        Ok(DBHandler { db })
    }
    async fn get_entries(&self) -> Result<Vec<DBEntry>> {
        let query = "SELECT id, name, path, parent FROM entries";
        Ok(sqlx::query_as::<_, DBEntry>(&query)
            .fetch_all(&self.db)
            .await?)
    }
    async fn add_entry(&self, e: &Entry) -> Result<()> {
        let dbe = DBEntry {
            id: e.id.to_string(),
            name: e.name.clone(),
            path: e.path.to_string_lossy().to_string(),
            parent: e.parent.to_string(),
        };

        let query = "INSERT INTO entries(id,name,path,parent) VALUES (?1, ?2, ?3, ?4)";
        sqlx::query(&query)
            .bind(dbe.id)
            .bind(dbe.name)
            .bind(dbe.path)
            .bind(dbe.parent)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    async fn add_subject(&self, s: &Subject) -> Result<()> {
        let dbs = DBSubject {
            id: s.id.to_string(),
            name: s.name.clone(),
            path: s.path.to_string_lossy().to_string(),
            icon: None,
        };
        let query = "INSERT INTO subjects(id, name, path, icon) VALUES (?1, ?2, ?3, ?4)";
        sqlx::query(&query)
            .bind(dbs.id)
            .bind(dbs.name)
            .bind(dbs.path)
            .bind(dbs.icon)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    async fn get_subjects(&self) -> Result<Vec<DBSubject>> {
        let query = "SELECT id, name, path, icon FROM subjects";
        Ok(sqlx::query_as::<_, DBSubject>(&query)
            .fetch_all(&self.db)
            .await?)
    }

}

struct DataHandler {
    dbh: DBHandler,
    subjects: HashMap<Uuid, Subject>,
    entries: HashMap<Uuid, Entry>,
}

impl DataHandler {
    fn read_dir(
        entries: &mut HashMap<PathBuf, DBEntry>,
        output: &mut HashMap<Uuid, Entry>,
        pending_entries: &mut Vec<Entry>,
        dir: &Path,
        dbh: &DBHandler,
        subject: &Subject,
    ) {
        if !dir.is_dir() {
            return;
        }

        if let Some(dir_name) = dir.file_name() {
            let dir_name = dir_name.to_string_lossy().to_string();
            if dir_name.starts_with(".") {
                return;
            }
        } else {
            return;
        }

        for entry in fs::read_dir(dir).unwrap().into_iter() {
            let entry = entry.unwrap();
            let entry_path = entry.path();
            if !entry_path.exists() {
                continue;
            }

            if let Some(entry_name) = entry_path.file_name() {
                let entry_name = entry_name.to_string_lossy().to_string();
                if entry_name.starts_with(".") {
                    continue;
                }
            } else {
                continue;
            }

            if entry_path.is_file() {
                let entry = match entries.get(&entry_path) {
                    Some(dbe) => {
                        let e = Entry::from_db(dbe, true);
                        entries.remove(&e.path);
                        e
                    }
                    None => {
                        let e = Entry::new(entry_path, subject.id, true);
                        pending_entries.push(e.clone());
                        e
                    }
                };
                output.insert(entry.id, entry);
            } else if entry_path.is_dir() {
                Self::read_dir(entries, output, pending_entries, &entry_path, dbh, subject);
            }
        }
    }

    async fn init(dbh: DBHandler, home_dir: &Path) -> Self {
        let mut subject_map: HashMap<PathBuf, DBSubject> = dbh
            .get_subjects()
            .await
            .unwrap()
            .into_iter()
            .map(|s| (PathBuf::from(&s.path), s))
            .collect();
        let mut output_subjects = HashMap::new();

        let mut entry_map: HashMap<PathBuf, DBEntry> = dbh
            .get_entries()
            .await
            .unwrap()
            .into_iter()
            .map(|e| (PathBuf::from(&e.path), e))
            .collect();
        let mut output_entries: HashMap<Uuid, Entry> = HashMap::new();

        let mut pending_entries = Vec::new();
        for e in fs::read_dir(&home_dir).unwrap() {
            let e = e.unwrap();
            let path = e.path();

            if let Some(name) = path.file_name() {
                let name = name.to_string_lossy().to_string();
                if name.starts_with(".") {
                    continue;
                }
            } else {
                continue;
            }

            if path.is_dir() {
                // checks if dir name doesnt start with .
                let subject = subject_map.remove(&path);
                let subject = match subject {
                    Some(s) => Subject::from_db(&s, true),
                    None => {
                        let s = Subject::new(path.clone(), None, true);
                        dbh.add_subject(&s).await.unwrap();
                        s
                    }
                };

                Self::read_dir(
                    &mut entry_map,
                    &mut output_entries,
                    &mut pending_entries,
                    &subject.path,
                    &dbh,
                    &subject,
                );
                output_subjects.insert(subject.id, subject);
            }
        }

        for e in pending_entries {
            dbh.add_entry(&e).await.unwrap();
        }

        subject_map.into_iter().for_each(|(_, dbs)| {
            output_subjects.insert(
                Uuid::parse_str(&dbs.id).unwrap(),
                Subject::from_db(&dbs, false),
            );
        });

        entry_map.into_iter().for_each(|(_, dbe)| {
            output_entries.insert(
                Uuid::parse_str(&dbe.id).unwrap(),
                Entry::from_db(&dbe, false),
            );
        });

        DataHandler {
            dbh,
            subjects: output_subjects,
            entries: output_entries,
        }
    }

    pub fn get_all_entries(&self) -> Vec<Entry> {
        self.entries.iter().map(|(_, e)| e.clone()).collect()
    }

    pub fn get_all_subjects(&self) -> Vec<Subject> {
        self.subjects.iter().map(|(_, e)| e.clone()).collect()
    }
}

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

struct SKStateInner {
    data: Option<DataHandler>,
    c: Option<Config>,
}

type SKState = Mutex<SKStateInner>;

async fn setup(app: &AppHandle) -> Result<()> {
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
