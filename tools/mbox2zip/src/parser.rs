use anyhow::{anyhow, Result};
use base64::Engine;
use mail_parser::{MessageParser, MimeHeaders};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

static RE_RFC2047: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)=\?([^?]+)\?([QB])\?([^?]*)\?=").unwrap());
static RE_WS_BETWEEN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\?=\s+=\?").unwrap());
static RE_MALFORMED: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\?([^? ]+)\?([QB])\?([^? ]*)\?=").unwrap());
static RE_SCRIPT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)<script.*?>.*?</script>").unwrap());
static RE_STYLE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)<style.*?>.*?</style>").unwrap());
static RE_TAGS: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]*>").unwrap());

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
}

pub struct MboxIterator<R: Read> {
    reader: BufReader<R>,
    buffer: Vec<u8>,
}

impl<R: Read> MboxIterator<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            buffer: Vec::new(),
        }
    }
}

impl<R: Read> Iterator for MboxIterator<R> {
    type Item = Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut msg_buffer = Vec::new();
        let mut line = Vec::new();

        loop {
            line.clear();
            match self.reader.read_until(b'\n', &mut line) {
                Ok(0) => {
                    if msg_buffer.is_empty() {
                        return None;
                    } else {
                        return Some(Ok(msg_buffer));
                    }
                }
                Ok(_) => {
                    // MBOX "From " line always starts at the beginning of a line.
                    // Instead of converting every line to a string, check bytes directly.
                    if line.starts_with(b"From ") {
                        if msg_buffer.is_empty() {
                            msg_buffer.extend_from_slice(&line);
                        } else {
                            self.buffer = line.clone();
                            return Some(Ok(msg_buffer));
                        }
                    } else {
                        msg_buffer.extend_from_slice(&line);
                    }
                    if !self.buffer.is_empty() && msg_buffer.is_empty() {
                        msg_buffer.extend_from_slice(&self.buffer);
                        self.buffer.clear();
                    }
                }
                Err(e) => return Some(Err(anyhow!(e))),
            }
        }
    }
}

pub fn extract_metadata(eml_data: &[u8], id: String) -> Option<MetadataEntry> {
    // MBOX messages start with a "From " line. Let's skip it aggressively.
    let mut data = eml_data;
    while data.starts_with(b"From ") || data.starts_with(b"\n") || data.starts_with(b"\r\n") {
        if let Some(pos) = data.iter().position(|&b| b == b'\n') {
            data = &data[pos + 1..];
        } else {
            break;
        }
    }

    // Identify header section to avoid parsing entire body for headers
    let header_end = data
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|p| p + 4)
        .or_else(|| data.windows(2).position(|w| w == b"\n\n").map(|p| p + 2))
        .unwrap_or(data.len());

    let header_data = &data[..header_end];
    let message = MessageParser::default().parse(data)?;

    // Combined manual header extraction to avoid repeated scans
    let mut manual_headers: HashMap<String, String> = HashMap::new();
    let header_text = String::from_utf8_lossy(header_data);
    let mut current_header: Option<String> = None;
    let mut current_value = String::new();

    for line in header_text.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            if let Some(_) = current_header {
                current_value.push(' ');
                current_value.push_str(line.trim());
            }
        } else {
            if let Some(key) = current_header.take() {
                manual_headers.insert(key, current_value.split_off(0));
            }
            if let Some(pos) = line.find(':') {
                let key = line[..pos].to_lowercase();
                current_header = Some(key);
                current_value.push_str(line[pos + 1..].trim());
            }
        }
    }
    if let Some(key) = current_header {
        manual_headers.insert(key, current_value);
    }

    let get_best_header = |name: &str| -> Option<String> {
        message
            .header(name)
            .and_then(|h| h.as_text().map(|t| decode_header_robust(t)))
            .or_else(|| {
                manual_headers
                    .get(&name.to_lowercase())
                    .map(|v| decode_header_robust(v))
            })
    };

    let subject = get_best_header("Subject");

    // Address extraction function
    fn get_address(s: &str) -> String {
        if let Some(start) = s.find('<') {
            if let Some(end) = s[start..].find('>') {
                return s[start + 1..start + end].to_string();
            }
        }
        s.trim().to_string()
    }

    let sender_name = get_best_header("From");
    let sender_address = sender_name.as_ref().map(|s| get_address(s));

    let to_raw = get_best_header("To");
    let to_addresses = to_raw.map(|t| {
        t.split(',')
            .map(|s| get_address(s.trim()))
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>()
    });

    let cc_raw = get_best_header("Cc");
    let cc_addresses = cc_raw.map(|t| {
        t.split(',')
            .map(|s| get_address(s.trim()))
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>()
    });

    let date_obj = message.date();
    let date_sent_iso = date_obj.map(|d| d.to_rfc3339());
    let internal_date = date_sent_iso.clone();

    let gmail_labels = get_best_header("X-Gmail-Labels").map(|decoded| {
        decoded
            .split(',')
            .map(|s| {
                let s = s.trim();
                let mut sanitized = s.chars().filter(|c| !c.is_control()).collect::<String>();
                if sanitized.contains("  ") {
                    sanitized = sanitized.replace("  ", " ");
                }
                sanitized.trim().to_string()
            })
            .filter(|s| {
                let lower = s.to_lowercase();
                !s.is_empty() && !lower.starts_with("kategorie") && !lower.starts_with("forward to")
            })
            .collect::<Vec<String>>()
    });

    let mut attachments = Vec::new();
    for part in message.attachments() {
        attachments.push(AttachmentMetadata {
            filename: part.attachment_name().map(|s| s.to_string()),
            mime: part
                .content_type()
                .as_ref()
                .map(|c| c.c_type.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_string()),
            size: part.contents().len(),
            content_id: part.content_id().map(|s| s.to_string()),
        });
    }

    let has_attachment = !attachments.is_empty();

    let body_text = message
        .body_html(0)
        .map(|html| strip_html(&html))
        .or_else(|| message.body_text(0).map(|t| t.to_string()));

    let snippet = body_text.map(|text: String| {
        let cleaned = text.replace('\n', " ").replace('\r', "").trim().to_string();
        if cleaned.chars().count() > 150 {
            let truncated: String = cleaned.chars().take(150).collect();
            format!("{}...", truncated)
        } else {
            cleaned
        }
    });

    Some(MetadataEntry {
        id,
        subject,
        sender_name,
        sender_address,
        to_addresses,
        cc_addresses,
        date_sent_iso,
        internal_date,
        gmail_labels,
        rfc822_size: eml_data.len(),
        message_id: message.message_id().map(|s: &str| s.to_string()),
        has_attachment,
        snippet,
        attachments: if attachments.is_empty() {
            None
        } else {
            Some(attachments)
        },
    })
}

fn strip_html(html: &str) -> String {
    let text = RE_SCRIPT.replace_all(html, "");
    let text = RE_STYLE.replace_all(&text, "");
    let text = RE_TAGS.replace_all(&text, " ");

    let text = text
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"");

    text.to_string()
}

pub fn decode_header_robust(value: &str) -> String {
    let unfolded = value.replace("\r\n", "").replace('\n', "");
    let collapsed = RE_WS_BETWEEN.replace_all(&unfolded, "?==?");

    let mut result = RE_RFC2047
        .replace_all(&collapsed, |caps: &regex::Captures| {
            let encoding = caps.get(2).unwrap().as_str().to_uppercase();
            let data = caps.get(3).unwrap().as_str();

            match encoding.as_str() {
                "Q" => {
                    let q_data = data.replace('_', " ");
                    match quoted_printable::decode(q_data, quoted_printable::ParseMode::Robust) {
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

    if RE_MALFORMED.is_match(&result) {
        result = RE_MALFORMED
            .replace_all(&result, |caps: &regex::Captures| {
                let encoding = caps.get(2).unwrap().as_str().to_uppercase();
                let data = caps.get(3).unwrap().as_str();

                match encoding.as_str() {
                    "Q" => {
                        let q_data = data.replace('_', " ");
                        match quoted_printable::decode(q_data, quoted_printable::ParseMode::Robust)
                        {
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
