use rusqlite::{params, Connection, Result};
use crate::parser::MetadataEntry;

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE messages (
            id TEXT PRIMARY KEY,
            subject TEXT,
            sender_name TEXT,
            sender_address TEXT,
            date_sent_iso TEXT,
            has_attachment INTEGER,
            labels TEXT
        )",
        [],
    )?;

    conn.execute("CREATE INDEX idx_messages_date ON messages(date_sent_iso)", [])?;

    conn.execute(
        "CREATE VIRTUAL TABLE messages_fts USING fts5(
            id UNINDEXED,
            subject,
            sender_name,
            sender_address,
            recipients,
            snippet,
            attachment_names
        )",
        [],
    )?;

    Ok(())
}

pub fn insert_metadata(conn: &Connection, entry: &MetadataEntry) -> Result<()> {
    let recipients = entry.to_addresses.as_ref().map(|v| v.join(" ")).unwrap_or_default() + " " + 
                     &entry.cc_addresses.as_ref().map(|v| v.join(" ")).unwrap_or_default();
    
    let att_names = entry.attachments.as_ref().map(|v| {
        v.iter().filter_map(|a| a.filename.clone()).collect::<Vec<_>>().join(" ")
    }).unwrap_or_default();

    let labels_str = entry.gmail_labels.as_ref().map(|v| v.join(" ")).unwrap_or_default();

    {
        let mut stmt = conn.prepare_cached(
            "INSERT INTO messages (id, subject, sender_name, sender_address, date_sent_iso, has_attachment, labels)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )?;
        stmt.execute(params![
            entry.id,
            entry.subject,
            entry.sender_name,
            entry.sender_address,
            entry.date_sent_iso,
            if entry.has_attachment { 1 } else { 0 },
            labels_str
        ])?;
    }

    {
        let mut stmt = conn.prepare_cached(
            "INSERT INTO messages_fts (id, subject, sender_name, sender_address, recipients, snippet, attachment_names)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )?;
        stmt.execute(params![
            entry.id,
            entry.subject,
            entry.sender_name,
            entry.sender_address,
            recipients.trim(),
            entry.snippet,
            att_names
        ])?;
    }

    Ok(())
}
