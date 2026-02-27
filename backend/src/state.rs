use crate::model::MetadataEntry;
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs::File;
use std::sync::{Arc, Mutex};
use zip::ZipArchive;

use crate::settings::Settings;

use std::sync::atomic::AtomicBool;

#[derive(Clone, Default, serde::Serialize)]
pub struct ConversionStatus {
    pub progress_percent: u8,
    pub current_message: u64,
    pub total_bytes: u64,
    pub total_bytes_read: u64,
    pub is_running: bool,
    pub error: Option<String>,
}

pub struct AppData {
    pub settings: Settings,
    pub metadata: Vec<MetadataEntry>,
    pub metadata_index: HashMap<String, usize>,
    pub labels: Vec<String>,
    pub db_conn: Option<Connection>,
    pub zip_archive: Option<ZipArchive<File>>,
}

#[derive(Clone)]
pub struct AppState {
    pub data: Arc<Mutex<AppData>>,
    pub instance_id: Arc<Mutex<String>>,
    pub conversion_status: Arc<Mutex<ConversionStatus>>,
    pub conversion_abort: Arc<AtomicBool>,
    pub is_loading: Arc<AtomicBool>,
    pub log_tx: Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<String>>>>,
}

impl AppState {
    pub fn new(
        settings: Settings,
        metadata: Vec<MetadataEntry>,
        db_conn: Option<Connection>,
        zip_archive: Option<ZipArchive<File>>,
        log_tx: Option<tokio::sync::mpsc::UnboundedSender<String>>,
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

        let data = AppData {
            settings,
            metadata,
            metadata_index: index,
            labels,
            db_conn,
            zip_archive,
        };

        Self {
            data: Arc::new(Mutex::new(data)),
            instance_id: Arc::new(Mutex::new(instance_id)),
            conversion_status: Arc::new(Mutex::new(ConversionStatus::default())),
            conversion_abort: Arc::new(AtomicBool::new(false)),
            is_loading: Arc::new(AtomicBool::new(true)),
            log_tx: Arc::new(Mutex::new(log_tx)),
        }
    }

    /// Hot-reloads data from a new settings object without restarting the server.
    pub fn apply_new_data(
        &self,
        new_settings: Settings,
        new_metadata: Vec<MetadataEntry>,
        new_db_conn: Option<Connection>,
        new_zip_archive: Option<ZipArchive<File>>,
    ) {
        let mut index = HashMap::new();
        let mut label_set = std::collections::HashSet::new();

        for (i, entry) in new_metadata.iter().enumerate() {
            index.insert(entry.id.clone(), i);
            if let Some(entry_labels) = &entry.gmail_labels {
                for label in entry_labels {
                    label_set.insert(label.clone());
                }
            }
        }

        let mut labels: Vec<String> = label_set.into_iter().collect();
        labels.sort();

        // Swap everything under one lock
        let mut data = self.data.lock().unwrap();
        data.settings = new_settings;
        data.metadata = new_metadata;
        data.metadata_index = index;
        data.labels = labels;
        data.db_conn = new_db_conn;
        data.zip_archive = new_zip_archive;

        // Update instance_id to trigger frontend notifications if needed
        *self.instance_id.lock().unwrap() = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string();
    }
}
