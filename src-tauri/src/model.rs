use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug)]
pub struct DBEntry {
    pub id: String,
    pub name: String,
    pub path: String,
    pub parent: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entry {
    pub id: uuid::Uuid,
    pub name: String,
    pub path: PathBuf,
    pub parent: uuid::Uuid,
    pub exists: bool,
}

impl Entry {
    pub fn from_db(s: &DBEntry, exists: bool) -> Self {
        let path = PathBuf::from(&s.path);
        Self {
            id: Uuid::parse_str(&s.id).unwrap(),
            name: s.name.clone(),
            path,
            parent: Uuid::parse_str(&s.parent).unwrap(),
            exists,
        }
    }
    pub fn new(path: PathBuf, parent_id: Uuid, exists: bool) -> Self {
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
pub struct DBSubject {
    pub id: String,
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Subject {
    pub id: Uuid,
    pub name: String,
    pub path: PathBuf,
    pub icon: Option<String>,
    pub exists: bool,
}

impl Subject {
    pub fn new(path: PathBuf, icon: Option<String>, exists: bool) -> Self {
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
    pub fn from_db(s: &DBSubject, exists: bool) -> Self {
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
pub struct DBTagAlias {
    pub id: String,
    pub alias: String,
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TagAlias {
    pub id: String,
    pub alias: String,
    pub tag: String,
}

#[derive(sqlx::FromRow, Debug)]
pub struct DBTagRelation {
    pub id: String,
    pub parent: String,
    pub child: String,
}

#[derive(sqlx::FromRow, Debug)]
pub struct DBTag {
    pub id: String,
    pub name: String,
    pub color: String,
    pub icon: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: uuid::Uuid,
    pub name: String,
    pub color: String,
    pub icon: String,
    pub aliases: Vec<String>,
    pub parent: Uuid,
    pub children: Vec<Uuid>,
}
