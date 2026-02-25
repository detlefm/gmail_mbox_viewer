use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentMetadata {
    pub filename: Option<String>,
    pub mime: String,
    pub size: usize,
    pub content_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataEntry {
    pub id: String,
    pub subject: Option<String>,
    pub sender_name: Option<String>,
    pub sender_address: Option<String>,
    pub to_addresses: Option<Vec<String>>,
    pub cc_addresses: Option<Vec<String>>,
    pub date_sent_iso: Option<String>,
    pub internal_date: Option<String>,
    pub gmail_labels: Option<Vec<String>>,
    pub rfc822_size: usize,
    pub message_id: Option<String>,
    pub has_attachment: bool,
    pub snippet: Option<String>,
    pub attachments: Option<Vec<AttachmentMetadata>>,
    // Add other fields as necessary, matching metadata.json
}
