pub mod db;
pub mod parser;

use crate::parser::{extract_metadata, MboxIterator};
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use zip::write::FileOptions;
use zip::ZipWriter;

/// Converts an MBOX file to an MBXC (ZIP-based) archive.
/// 
/// `progress_callback` receives (total_bytes_read_so_far, current_message_count).
pub fn convert_mbox_to_mbxc(
    input_path: PathBuf,
    output_path: PathBuf,
    progress_callback: Option<Box<dyn Fn(u64, u64) + Send>>,
) -> Result<()> {
    let input_file = File::open(&input_path).context("Failed to open input MBOX")?;
    let mbox_iter = MboxIterator::new(input_file);

    let zip_file = File::create(&output_path).context("Failed to create output MBXC archive")?;
    let mut zip = ZipWriter::new(BufWriter::with_capacity(1024 * 1024, zip_file));

    // Temp DB
    let temp_db_path = tempfile::Builder::new().suffix(".db").tempfile()?;
    let mut conn = Connection::open(temp_db_path.path())?;
    
    // Performance pragmas for SQLite
    conn.execute_batch(
        "PRAGMA journal_mode = OFF;
         PRAGMA synchronous = OFF;
         PRAGMA cache_size = 100000;
         PRAGMA locking_mode = EXCLUSIVE;
         PRAGMA temp_store = MEMORY;"
    )?;

    db::init_db(&conn)?;

    let mut metadata_entries = Vec::new();

    // Use a single transaction for the entire process for maximum speed
    let tx = conn.transaction()?;

    let mut cumulative_bytes = 0;
    let mut total_count = 0;

    for (idx, msg_bytes_res) in mbox_iter.enumerate() {
        let msg_bytes = msg_bytes_res?;
        cumulative_bytes += msg_bytes.len() as u64;
        
        let msg_idx = (idx + 1) as u64;
        total_count = msg_idx;
        let id = format!("msg_{:06}.eml", msg_idx);

        if let Some(meta) = extract_metadata(&msg_bytes, id.clone()) {
            db::insert_metadata(&tx, &meta)?;
            metadata_entries.push(meta);

            let options = FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o644);
            
            zip.start_file(&id, options)?;
            zip.write_all(&msg_bytes)?;
        }

        // Report progress every 500 messages
        if msg_idx % 500 == 0 {
            if let Some(ref cb) = progress_callback {
                cb(cumulative_bytes, msg_idx);
            }
        }
    }

    // Final progress update
    if let Some(ref cb) = progress_callback {
        cb(cumulative_bytes, total_count);
    }

    // Commit the DB transaction
    tx.commit()?;

    // Write JSON metadata to MBXC
    zip.start_file("metadata.json", FileOptions::default())?;
    serde_json::to_writer(&mut zip, &metadata_entries)?;

    // Close DB connection and write to ZIP
    drop(conn); // ensure flushed

    zip.start_file("metadata.db", FileOptions::default())?;
    let mut db_file = File::open(temp_db_path.path())?;
    std::io::copy(&mut db_file, &mut zip)?;

    zip.finish()?;

    Ok(())
}
