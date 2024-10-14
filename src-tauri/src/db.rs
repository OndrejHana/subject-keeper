use crate::{config::DB_FILENAME, error::*, model::{DBEntry, DBSubject, Entry, Subject}};
use std::{fs::{self, create_dir_all, OpenOptions}, path::{Path, PathBuf}};

use sqlx::{sqlite::SqlitePoolOptions, Executor, SqlitePool};

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


#[derive(Debug)]
pub struct DBHandler {
    pub db: SqlitePool,
}

impl DBHandler {
    pub async fn new(db_dir: &Path) -> Result<Self> {
        create_dir_all(&db_dir)?;
        let mut db_path = PathBuf::from(db_dir);
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

    pub async fn load(db_dir: &Path) -> Result<Self> {
        let db = SqlitePoolOptions::new()
            .connect(db_dir.to_str().unwrap())
            .await?;
        Ok(DBHandler { db })
    }

    pub fn exists(db_dir: &Path) -> bool {
        let mut path = PathBuf::from(db_dir);
        path.push(DB_FILENAME);
        path.exists()
    }

    pub async fn get_entries(&self) -> Result<Vec<DBEntry>> {
        let query = "SELECT id, name, path, parent FROM entries";
        Ok(sqlx::query_as::<_, DBEntry>(&query)
            .fetch_all(&self.db)
            .await?)
    }

    pub async fn add_entry(&self, e: &Entry) -> Result<()> {
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

    pub async fn add_subject(&self, s: &Subject) -> Result<()> {
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

    pub async fn get_subjects(&self) -> Result<Vec<DBSubject>> {
        let query = "SELECT id, name, path, icon FROM subjects";
        Ok(sqlx::query_as::<_, DBSubject>(&query)
            .fetch_all(&self.db)
            .await?)
    }

}
