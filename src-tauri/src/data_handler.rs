use std::{collections::HashMap, fs::read_dir, path::{Path, PathBuf}};

use uuid::Uuid;

use crate::{db::DBHandler, model::{DBEntry, DBSubject, Entry, Subject}};

pub struct DataHandler {
    pub dbh: DBHandler,
    pub subjects: HashMap<Uuid, Subject>,
    pub entries: HashMap<Uuid, Entry>,
}

impl DataHandler {
    pub fn read_dir(
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

        for entry in read_dir(dir).unwrap().into_iter() {
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

    pub async fn init(dbh: DBHandler, home_dir: &Path) -> Self {
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
        for e in read_dir(&home_dir).unwrap() {
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
