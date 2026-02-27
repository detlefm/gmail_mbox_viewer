use crate::model::MetadataEntry;
use crate::state::AppState;
use axum::{
    extract::{Path as AxumPath, State, Query},
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
} ;
use std::sync::atomic::Ordering;
use base64::Engine;
use mail_parser::{MessageParser, MimeHeaders};
use once_cell::sync::Lazy;
use regex::Regex;
use rfd::FileDialog;
use serde::Deserialize;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::fs;

static RE_RFC2047: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)=\?([^?]+)\?([QB])\?([^?]*)\?=").unwrap());
static RE_WS_BETWEEN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\?=\s+=\?").unwrap());
static RE_MALFORMED: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\?([^? ]+)\?([QB])\?([^? ]*)\?=").unwrap());

// --- DTOs ---

#[derive(Deserialize)]
pub struct SearchQuery {
    pub any: Option<String>,
    pub sender: Option<String>,
    pub subject: Option<String>,
    pub label: Option<String>,
    pub has_attachment: Option<bool>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(serde::Serialize)]
pub struct SearchResult {
    total: usize,
    messages: Vec<MetadataEntry>,
}

// --- Handlers ---

pub async fn get_labels(State(state): State<AppState>) -> impl IntoResponse {
    if state.is_loading.load(Ordering::SeqCst) {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    let data = state.data.lock().unwrap();
    let filter_labels = data.settings.filter_labels.as_ref();

    let mut filtered_labels: Vec<String> = data
        .labels
        .iter()
        .filter(|&label| {
            if let Some(filters) = filter_labels {
                !filters.contains(label)
            } else {
                true
            }
        })
        .cloned()
        .collect();

    if !filtered_labels.iter().any(|l| l == "Alle Mails") {
        filtered_labels.insert(0, "Alle Mails".to_string());
    }

    Json(filtered_labels).into_response()
}

pub async fn search_messages(
    State(state): State<AppState>,
    Json(query): Json<SearchQuery>,
) -> impl IntoResponse {
    if state.is_loading.load(Ordering::SeqCst) {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let data = state.data.lock().unwrap();

    // In-Memory Search (Fallback)
    // Filter metadata
    let filtered: Vec<&MetadataEntry> = data
        .metadata
        .iter()
        .filter(|entry| {
            // Basis-Filter für spezielle Labels (Spam, Papierkorb, Gesendet)
            // Diese werden grundsätzlich ausgeschlossen, außer der User hat explizit
            // eines dieser Labels in der Sidebar oder im Label-Dropdown ausgewählt.
            let searching_special = query
                .label
                .as_ref()
                .map(|l| {
                    data.settings
                        .special_labels
                        .as_ref()
                        .map(|s| s.contains(l))
                        .unwrap_or(false)
                })
                .unwrap_or(false);

            if !searching_special {
                if let Some(entry_labels) = &entry.gmail_labels {
                    if let Some(special) = &data.settings.special_labels {
                        if entry_labels.iter().any(|l| special.contains(l)) {
                            return false;
                        }
                    }
                }
            }

            // Label Filter
            if let Some(label) = &query.label {
                if !label.is_empty() && label != "Alle Mails" {
                    if let Some(labels) = &entry.gmail_labels {
                        if !labels.contains(label) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
            }

            // Subject Filter
            if let Some(subject) = &query.subject {
                if !subject.is_empty() {
                    let q = subject.to_lowercase();
                    if !entry
                        .subject
                        .as_ref()
                        .map(|s| s.to_lowercase().contains(&q))
                        .unwrap_or(false)
                    {
                        return false;
                    }
                }
            }

            // Attachment Filter
            if let Some(has_att) = query.has_attachment {
                if has_att && !entry.has_attachment {
                    return false;
                }
            }

            // Date Filters
            if let Some(from) = &query.date_from {
                if !from.is_empty() {
                    if let Some(entry_date) = &entry.date_sent_iso {
                        if entry_date < from {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
            }
            if let Some(to) = &query.date_to {
                if !to.is_empty() {
                    if let Some(entry_date) = &entry.date_sent_iso {
                        if entry_date > to {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
            }

            // Any Filter (Full Text Search - Subject/Sender)
            if let Some(any) = &query.any {
                if !any.is_empty() {
                    let q = any.to_lowercase();
                    let subject_match = entry
                        .subject
                        .as_ref()
                        .map(|s| s.to_lowercase().contains(&q))
                        .unwrap_or(false);
                    let sender_match = entry
                        .sender_name
                        .as_ref()
                        .map(|s| s.to_lowercase().contains(&q))
                        .unwrap_or(false)
                        || entry
                            .sender_address
                            .as_ref()
                            .map(|s| s.to_lowercase().contains(&q))
                            .unwrap_or(false);
                    if !subject_match && !sender_match {
                        return false;
                    }
                }
            }

            // Sender Filter
            if let Some(sender) = &query.sender {
                if !sender.is_empty() {
                    let q = sender.to_lowercase();
                    let sender_match = entry
                        .sender_name
                        .as_ref()
                        .map(|s| s.to_lowercase().contains(&q))
                        .unwrap_or(false)
                        || entry
                            .sender_address
                            .as_ref()
                            .map(|s| s.to_lowercase().contains(&q))
                            .unwrap_or(false);
                    if !sender_match {
                        return false;
                    }
                }
            }

            true
        })
        .collect();

    let total = filtered.len();
    let mut paged: Vec<MetadataEntry> = filtered
        .into_iter()
        .skip(offset)
        .take(limit)
        .cloned()
        .collect();

    // Labels für die Anzeige filtern (filter_labels beachten)
    if let Some(filters) = &data.settings.filter_labels {
        for entry in &mut paged {
            if let Some(labels) = &mut entry.gmail_labels {
                labels.retain(|l| !filters.contains(l));
            }
        }
    }

    Json(SearchResult {
        total,
        messages: paged,
    })
    .into_response()
}

pub async fn get_message(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
) -> Result<Response, StatusCode> {
    if state.is_loading.load(Ordering::SeqCst) {
        return Ok(StatusCode::SERVICE_UNAVAILABLE.into_response());
    }
    let mut data = state.data.lock().unwrap();

    let (body, is_html, attachments) = {
        let archive = data.zip_archive.as_mut().ok_or(StatusCode::NOT_FOUND)?;

        let mut eml_file = archive.by_name(&id).map_err(|_| StatusCode::NOT_FOUND)?;
        let mut buffer = Vec::new();
        eml_file
            .read_to_end(&mut buffer)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Parse EML
        let message = MessageParser::default()
            .parse(&buffer)
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // Extract body (HTML/Text)
        let body = message
            .body_html(0)
            .map(|s| s.into_owned())
            .or_else(|| message.body_text(0).map(|s| s.into_owned()))
            .unwrap_or_else(|| "[Kein Inhalt]".to_string());

        let is_html = message.body_html(0).is_some();

        // Create JSON for attachments
        let attachments: Vec<serde_json::Value> = message
            .attachments()
            .map(|a| {
                serde_json::json!({
                    "filename": a.attachment_name().unwrap_or("unnamed"),
                    "content_type": a.content_type().as_ref().map(|c| c.c_type.to_string()),
                    "content_id": a.content_id().as_ref()
                })
            })
            .collect();

        (body, is_html, attachments)
    };

    // Release mutable borrow of data by using fields directly
    let entry_idx = data.metadata_index.get(&id);
    let entry = entry_idx.and_then(|&idx| data.metadata.get(idx));

    let mut labels = entry
        .and_then(|m| m.gmail_labels.clone())
        .unwrap_or_default();

    // Filter labels to hide configured ones
    if let Some(filters) = &data.settings.filter_labels {
        labels.retain(|l| !filters.contains(l));
    }

    // Use metadata entries as preferred source for these fields for consistency and reliability
    let from_str = entry
        .map(|m| match (&m.sender_name, &m.sender_address) {
            (Some(n), Some(a)) => {
                if n.is_empty() {
                    a.clone()
                } else {
                    format!("{} <{}>", n, a)
                }
            }
            (None, Some(a)) => a.clone(),
            (Some(n), None) => n.clone(),
            _ => "".to_string(),
        })
        .unwrap_or_default();

    let to_str = entry
        .and_then(|m| m.to_addresses.as_ref().map(|v| v.join(", ")))
        .unwrap_or_default();
    let date_str = entry
        .and_then(|m| m.date_sent_iso.clone())
        .unwrap_or_default();
    let subject_str = entry.and_then(|m| m.subject.clone()).unwrap_or_default();

    Ok(Json(serde_json::json!({
        "id": id,
        "subject": decode_header_robust(&subject_str),
        "from": decode_header_robust(&from_str),
        "to": decode_header_robust(&to_str),
        "date": date_str,
        "body": body,
        "is_html": is_html,
        "attachments": attachments,
        "labels": labels,
        "gmail_labels": labels
    }))
    .into_response())
}

pub async fn download_attachment(
    State(state): State<AppState>,
    AxumPath((id, filename)): AxumPath<(String, String)>,
) -> Response {
    if state.is_loading.load(Ordering::SeqCst) {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    let mut data = match state.data.lock() {
        Ok(d) => d,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let archive = match data.zip_archive.as_mut() {
        Some(a) => a,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let mut eml_file = match archive.by_name(&id) {
        Ok(f) => f,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    let mut buffer = Vec::new();
    if eml_file.read_to_end(&mut buffer).is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let message = match MessageParser::default().parse(&buffer) {
        Some(m) => m,
        None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    for attachment in message.attachments() {
        let att_name = attachment.attachment_name().unwrap_or("");
        if att_name == filename {
            let content_type = attachment
                .content_type()
                .as_ref()
                .map(|c| c.c_type.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_string());
            let att_data = attachment.contents().to_vec();

            return (
                [
                    (header::CONTENT_TYPE, content_type),
                    (
                        header::CONTENT_DISPOSITION,
                        format!("attachment; filename=\"{}\"", filename),
                    ),
                ],
                att_data,
            )
                .into_response();
        }
    }

    StatusCode::NOT_FOUND.into_response()
}

pub fn decode_header_robust(value: &str) -> String {
    // 1. Unfold: Headers can have newlines followed by spaces.
    let unfolded = value.replace("\r\n", "").replace('\n', "");

    // 2. RFC 2047: White space between 'encoded-word's is ignored.
    let collapsed = RE_WS_BETWEEN.replace_all(&unfolded, "?==?");

    let mut result = collapsed.to_string();

    // 3. Decode standard RFC 2047: =?charset?encoding?data?=
    // Decode all valid MIME words
    result = RE_RFC2047
        .replace_all(&result, |caps: &regex::Captures| {
            let encoding = caps.get(2).unwrap().as_str().to_uppercase();
            let data = caps.get(3).unwrap().as_str();

            match encoding.as_str() {
                "Q" => {
                    let q_data = data.replace('_', " ");
                    let decoded =
                        quoted_printable::decode(q_data, quoted_printable::ParseMode::Robust);
                    match decoded {
                        Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                        Err(_) => caps.get(0).unwrap().as_str().to_string(),
                    }
                }
                "B" => match base64::engine::general_purpose::STANDARD.decode(data) {
                    Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                    Err(_) => caps.get(0).unwrap().as_str().to_string(),
                },
                _ => caps.get(0).unwrap().as_str().to_string(),
            }
        })
        .to_string();

    // 4. Fallback for malformed ones (missing leading =)
    if RE_MALFORMED.is_match(&result) {
        result = RE_MALFORMED
            .replace_all(&result, |caps: &regex::Captures| {
                let encoding = caps.get(2).unwrap().as_str().to_uppercase();
                let data = caps.get(3).unwrap().as_str();

                match encoding.as_str() {
                    "Q" => {
                        let q_data = data.replace('_', " ");
                        let decoded =
                            quoted_printable::decode(q_data, quoted_printable::ParseMode::Robust);
                        match decoded {
                            Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                            Err(_) => caps.get(0).unwrap().as_str().to_string(),
                        }
                    }
                    "B" => match base64::engine::general_purpose::STANDARD.decode(data) {
                        Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                        Err(_) => caps.get(0).unwrap().as_str().to_string(),
                    },
                    _ => caps.get(0).unwrap().as_str().to_string(),
                }
            })
            .to_string();
    }
    result
}

pub async fn get_system_info(State(state): State<AppState>) -> Json<serde_json::Value> {
    let data = state.data.lock().unwrap();
    let instance_id = state.instance_id.lock().unwrap();

    Json(serde_json::json!({
        "instance_id": *instance_id,
        "zip_path": data.settings.zip_path,
        "db_loaded": data.db_conn.is_some(),
        "is_loading": state.is_loading.load(Ordering::SeqCst),
        "settings_path": data.settings.source_path.as_ref().map(|p| p.to_string_lossy().to_string()),
        "browser": data.settings.browser,
        "os": std::env::consts::OS,
    }))
}

pub async fn select_file() -> Json<serde_json::Value> {
    let file = tokio::task::spawn_blocking(|| {
        FileDialog::new()
            .add_filter("MBOX or MBXC", &["mbox", "mbxc"])
            .pick_file()
    })
    .await
    .unwrap_or(None);

    Json(serde_json::json!({
        "path": file.map(|p| p.to_string_lossy().to_string())
    }))
}

pub async fn select_save_file() -> Json<serde_json::Value> {
    let file = tokio::task::spawn_blocking(|| {
        FileDialog::new()
            .add_filter("MBXC Archive", &["mbxc"])
            .save_file()
    })
    .await
    .unwrap_or(None);

    Json(serde_json::json!({
        "path": file.map(|p| p.to_string_lossy().to_string())
    }))
}

pub async fn select_toml_file() -> Json<serde_json::Value> {
    let file = tokio::task::spawn_blocking(|| {
        FileDialog::new()
            .add_filter("Settings", &["toml"])
            .pick_file()
    })
    .await
    .unwrap_or(None);

    Json(serde_json::json!({
        "path": file.map(|p| p.to_string_lossy().to_string())
    }))
}

pub async fn select_toml_save_file() -> Json<serde_json::Value> {
    let file = tokio::task::spawn_blocking(|| {
        FileDialog::new()
            .add_filter("Settings", &["toml"])
            .save_file()
    })
    .await
    .unwrap_or(None);

    Json(serde_json::json!({
        "path": file.map(|p| p.to_string_lossy().to_string())
    }))
}

// --- Custom Svelte File Explorer API ---

#[derive(serde::Serialize)]
pub struct FsEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
}

#[derive(Deserialize)]
pub struct ListDirQuery {
    pub path: String,
    pub show_files: Option<bool>,
    pub filter: Option<String>, // comma separated extensions like "mbox,mbxc"
}

pub async fn list_drives() -> Json<Vec<FsEntry>> {
    tokio::task::spawn_blocking(|| {
        let mut drives = Vec::new();
        #[cfg(windows)]
        {
            for drive_letter in b'A'..=b'Z' {
                let drive_path = format!("{}:\\", drive_letter as char);
                if Path::new(&drive_path).exists() {
                    drives.push(FsEntry {
                        name: drive_path.clone(),
                        path: drive_path,
                        is_dir: true,
                        size: None,
                    });
                }
            }
        }
        #[cfg(not(windows))]
        {
            drives.push(FsEntry {
                name: "/".to_string(),
                path: "/".to_string(),
                is_dir: true,
                size: None,
            });
        }
        Json(drives)
    })
    .await
    .unwrap_or(Json(Vec::new()))
}

pub async fn list_dir(
    Query(query): Query<ListDirQuery>,
) -> Result<Json<Vec<FsEntry>>, (StatusCode, String)> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&query.path);
        if !path.exists() {
            return Err((StatusCode::NOT_FOUND, "Path does not exist".to_string()));
        }

        let mut entries = Vec::new();
        let read_dir = match fs::read_dir(path) {
            Ok(rd) => rd,
            Err(e) => return Err((StatusCode::FORBIDDEN, e.to_string())),
        };

        let filters: Vec<String> = query
            .filter
            .as_deref()
            .unwrap_or("")
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect();

        for entry_res in read_dir {
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
            };

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let is_dir = metadata.is_dir();
            let name = entry.file_name().to_string_lossy().to_string();
            
            // Hidden file skip (rough)
            if name.starts_with('.') && name.len() > 1 && !is_dir {
                continue;
            }

            if is_dir {
                entries.push(FsEntry {
                    name,
                    path: entry.path().to_string_lossy().to_string(),
                    is_dir: true,
                    size: None,
                });
            } else if query.show_files.unwrap_or(true) {
                let ext = entry
                    .path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                if filters.is_empty() || filters.contains(&ext) {
                    entries.push(FsEntry {
                        name,
                        path: entry.path().to_string_lossy().to_string(),
                        is_dir: false,
                        size: Some(metadata.len()),
                    });
                }
            }
        }

        // Sort: Dirs first, then name
        entries.sort_by(|a, b| {
            if a.is_dir != b.is_dir {
                b.is_dir.cmp(&a.is_dir)
            } else {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
        });

        Ok(Json(entries))
    })
    .await
    .unwrap_or(Err((StatusCode::INTERNAL_SERVER_ERROR, "Panic in spawn_blocking".to_string())))
}



#[derive(serde::Deserialize)]
pub struct ConvertRequest {
    pub mbox_path: String,
    pub mbxc_path: String,
}

pub async fn convert_mbox(
    State(state): State<AppState>,
    Json(req): Json<ConvertRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    {
        let mut status = state.conversion_status.lock().unwrap();
        if status.is_running {
            return Err((
                StatusCode::CONFLICT,
                "Konvertierung läuft bereits".to_string(),
            ));
        }
        status.is_running = true;
        status.progress_percent = 0;
        status.current_message = 0;
        status.error = None;
    }
    state.conversion_abort.store(false, Ordering::SeqCst);

    let mbox_path = PathBuf::from(req.mbox_path);
    let mbxc_path = PathBuf::from(req.mbxc_path);
    let state_clone = state.clone();

    tokio::task::spawn_blocking(move || {
        let status_arc = state_clone.conversion_status.clone();
        let abort_arc = state_clone.conversion_abort.clone();

        let callback = move |bytes_read: u64, total_bytes: u64, msg_count: u64| {
            let mut status = status_arc.lock().unwrap();
            status.current_message = msg_count;
            status.total_bytes = total_bytes;
            status.total_bytes_read = bytes_read;
            if total_bytes > 0 {
                status.progress_percent = (bytes_read * 100 / total_bytes) as u8;
            }
        };

        match mbox2zip::convert_mbox_to_mbxc(
            mbox_path,
            mbxc_path,
            Some(Box::new(callback)),
            abort_arc,
        ) {
            Ok(finished) => {
                let mut status = state_clone.conversion_status.lock().unwrap();
                status.is_running = false;
                if !finished {
                    status.error = Some("Abgebrochen".to_string());
                } else {
                    status.progress_percent = 100;
                }
            }
            Err(e) => {
                let mut status = state_clone.conversion_status.lock().unwrap();
                status.is_running = false;
                status.error = Some(e.to_string());
            }
        }
    });

    Ok(Json(serde_json::json!({ "status": "started" })))
}

pub async fn get_convert_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let status = state.conversion_status.lock().unwrap();
    Json(serde_json::json!(*status))
}

pub async fn abort_convert(State(state): State<AppState>) -> Json<serde_json::Value> {
    state.conversion_abort.store(true, Ordering::SeqCst);
    Json(serde_json::json!({ "status": "aborting" }))
}

#[derive(serde::Deserialize)]
pub struct SettingsUpdateRequest {
    pub zip_path: String,
    pub browser: Option<String>,
}

pub async fn update_settings(
    State(state): State<AppState>,
    Json(req): Json<SettingsUpdateRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut settings = {
        let data = state.data.lock().unwrap();
        data.settings.clone()
    };

    let settings_path = settings.source_path.clone().ok_or((
        StatusCode::BAD_REQUEST,
        "No settings file location found".to_string(),
    ))?;

    // Update values
    settings.zip_path = req.zip_path;
    settings.browser = req.browser;

    let toml_string = toml::to_string_pretty(&settings)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    std::fs::write(&settings_path, toml_string)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Hot reload
    match crate::load_all_data(Some(settings_path), None) {
        Ok(raw) => {
            state.apply_new_data(raw.settings.clone(), raw.metadata, raw.db_conn, raw.archive);

            // Notify launcher to update UI
            if let Some(tx) = state.log_tx.lock().unwrap().as_ref() {
                let _ = tx.send(format!("MBXC_PATH:{}", raw.settings.zip_path));
            }

            Ok(Json(serde_json::json!({ "status": "success" })))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[derive(serde::Deserialize)]
pub struct RestartRequest {
    pub settings_path: String,
}

pub async fn restart_with_settings(
    State(state): State<AppState>,
    Json(req): Json<RestartRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let next_path = PathBuf::from(req.settings_path);

    // Hot reload from new toml
    match crate::load_all_data(Some(next_path.clone()), None) {
        Ok(raw) => {
            state.apply_new_data(raw.settings.clone(), raw.metadata, raw.db_conn, raw.archive);

            // Notify launcher to persist the new path and update UI
            if let Some(tx) = state.log_tx.lock().unwrap().as_ref() {
                let _ = tx.send(format!("SETTINGS_PATH:{}", next_path.display()));
                let _ = tx.send(format!("MBXC_PATH:{}", raw.settings.zip_path));
            }

            Ok(Json(serde_json::json!({ "status": "success" })))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn inspect_settings(
    Json(req): Json<RestartRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let path = PathBuf::from(&req.settings_path);
    let settings = crate::settings::Settings::new(Some(path))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut zip_path = PathBuf::from(&settings.zip_path);
    if zip_path.is_relative() {
        if let Some(source_path) = &settings.source_path {
            if let Some(parent) = source_path.parent() {
                zip_path = parent.join(&settings.zip_path);
            }
        }
    }

    Ok(Json(serde_json::json!({
        "zip_path": zip_path.to_string_lossy(),
        "browser": settings.browser,
    })))
}

#[derive(serde::Deserialize)]
pub struct CreateSettingsRequest {
    pub toml_path: String,
    pub zip_path: String,
    pub filter_labels: Vec<String>,
    pub special_labels: Vec<String>,
}

pub async fn create_settings(
    State(state): State<AppState>,
    Json(req): Json<CreateSettingsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let settings = crate::settings::Settings {
        zip_path: req.zip_path,
        filter_labels: Some(req.filter_labels),
        special_labels: Some(req.special_labels),
        browser: None,
        source_path: Some(PathBuf::from(&req.toml_path)),
    };

    let toml_string = toml::to_string_pretty(&settings)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    std::fs::write(&req.toml_path, toml_string)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Now restart the app with these new settings
    restart_with_settings(
        State(state),
        Json(RestartRequest {
            settings_path: req.toml_path,
        }),
    )
    .await
}
