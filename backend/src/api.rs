use crate::model::MetadataEntry;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
};
use base64::Engine;
use mail_parser::{MessageParser, MimeHeaders};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use std::io::Read;

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

pub async fn get_labels(State(state): State<AppState>) -> Json<Vec<String>> {
    let filter_labels = state.settings.filter_labels.as_ref();

    let mut filtered_labels: Vec<String> = state
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

    Json(filtered_labels)
}

pub async fn search_messages(
    State(state): State<AppState>,
    Json(query): Json<SearchQuery>,
) -> Json<SearchResult> {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    // In-Memory Search (Fallback)
    // Filter metadata
    let filtered: Vec<&MetadataEntry> = state
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
                    state
                        .settings
                        .special_labels
                        .as_ref()
                        .map(|s| s.contains(l))
                        .unwrap_or(false)
                })
                .unwrap_or(false);

            if !searching_special {
                if let Some(entry_labels) = &entry.gmail_labels {
                    if let Some(special) = &state.settings.special_labels {
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
    if let Some(filters) = &state.settings.filter_labels {
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
}

pub async fn get_message(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Use cached ZIP
    let mut archive = state
        .zip_archive
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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

    // Find entry in metadata for fallbacks and labels
    let entry = state
        .metadata_index
        .get(&id)
        .and_then(|&idx| state.metadata.get(idx));

    let mut labels = entry
        .and_then(|m| m.gmail_labels.clone())
        .unwrap_or_default();

    // Filter labels to hide configured ones
    if let Some(filters) = &state.settings.filter_labels {
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

    Ok(Json(serde_json::json!({
        "id": id,
        "subject": message.subject().map(|s| decode_header_robust(&s.to_string())),
        "from": decode_header_robust(&from_str),
        "to": decode_header_robust(&to_str),
        "date": date_str,
        "body": body,
        "is_html": is_html,
        "attachments": attachments,
        "labels": labels,
        "gmail_labels": labels
    })))
}

pub async fn download_attachment(
    State(state): State<AppState>,
    Path((id, filename)): Path<(String, String)>,
) -> Response {
    // Use cached ZIP
    let mut archive = match state.zip_archive.lock() {
        Ok(a) => a,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
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
            let data = attachment.contents().to_vec();

            return (
                [
                    (header::CONTENT_TYPE, content_type),
                    (
                        header::CONTENT_DISPOSITION,
                        format!("attachment; filename=\"{}\"", filename),
                    ),
                ],
                data,
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
    Json(serde_json::json!({
        "instance_id": state.instance_id,
        "zip_path": state.zip_path.to_string_lossy(),
        "db_loaded": state.db_conn.lock().unwrap().is_some()
    }))
}
