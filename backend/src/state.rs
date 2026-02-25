use crate::model::MetadataEntry;
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use zip::ZipArchive;

use crate::settings::Settings;

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub zip_path: PathBuf,
    pub metadata: Arc<Vec<MetadataEntry>>,
    pub metadata_index: Arc<HashMap<String, usize>>,
    pub labels: Arc<Vec<String>>,
    pub db_conn: Arc<Mutex<Option<Connection>>>,
    pub db_path: Option<PathBuf>,
    pub instance_id: String,
    pub zip_archive: Arc<Mutex<ZipArchive<File>>>,
}

impl AppState {
    pub fn new(
        settings: Settings,
        zip_path: PathBuf,
        metadata: Vec<MetadataEntry>,
        db_conn: Option<Connection>,
        zip_archive: ZipArchive<File>,
    ) -> Self {
        let instance_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string();

        let mut index = HashMap::new();
        let mut label_set = std::collections::HashSet::new();

        for (i, entry) in metadata.iter().enumerate() {
            index.insert(entry.id.clone(), i);
            if let Some(entry_labels) = &entry.gmail_labels {
                for label in entry_labels {
                    label_set.insert(label.clone());
                }
            }
        }

        let mut labels: Vec<String> = label_set.into_iter().collect();
        labels.sort();

        Self {
            settings,
            zip_path,
            metadata: Arc::new(metadata),
            metadata_index: Arc::new(index),
            labels: Arc::new(labels),
            db_conn: Arc::new(Mutex::new(db_conn)),
            db_path: None,
            instance_id,
            zip_archive: Arc::new(Mutex::new(zip_archive)),
        }
    }
}
