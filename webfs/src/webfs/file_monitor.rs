use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use redb::{Database, TableDefinition};
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::time::Duration;
use tokio::time;
use tracing;
use lazy_static::lazy_static;
use quick_xml::Writer;

use docx_rs::{DocumentChild, TableCell};
use crate::models::{file_desc::FileDesc, files::{Config, Channel}};
use bincode::serialize;

const FILENAMES_TABLE: TableDefinition<&str, ()> = TableDefinition::new("filenames");
const FILEDESC_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("filedesc");

pub async fn start_file_monitor(db_path: &str, config: &Config, pattern: &str) -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::create(db_path)?;

    let channel = config.channels.get("en")
        .and_then(|m| m.get("videos-all"))
        .expect("No videos-all channel found in config");

    // Ensure the table exists
    {
        let txn = db.begin_write()?;
        txn.open_table(FILENAMES_TABLE)?;
        txn.open_table(FILEDESC_TABLE)?;
        txn.commit()?;
    }

    let regex = Regex::new(pattern)?;

    let start_date = std::env::var("RSS_DAYS").unwrap_or("7".to_string()).parse::<i32>().ok()
        .map(|days| Utc::now().date_naive() - chrono::Duration::days(days.abs() as i64));

    let channel = channel.clone();
    let regex = regex.clone();
    let config_clone = config.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5)); // Poll every 5 seconds
        loop {
            interval.tick().await;
            tracing::info!("Scanning files... {}", channel.file_path);
            if let Err(e) = scan_and_store(&db, &channel, &regex).await {
                tracing::error!("Error scanning files: {}", e);
            }
            if let Err(e) = write_rss(config_clone.channels.values().flat_map(|m| m.iter()).collect(), start_date){
                tracing::error!("Error writing RSS: {}", e);
            }
        }
    });

    Ok(())
}

fn write_rss(channels_to_process: Vec<(&String, &Channel)>, start_date: Option<NaiveDate>) -> Result<()> {
    tracing::info!("Processing {} channels", channels_to_process.len());
    for (channel_name, ch) in channels_to_process {

        let output_path = &ch.output_path;

        tracing::info!("---------------------------------------------------------");
        tracing::info!("Refreshing RSS Channel {} {}", channel_name, output_path);

        // Read and filter files from the directory
        let entries = Channel::read_dir(&ch)?;
        if entries.is_empty() {
            tracing::warn!("No entries found for channel {}", channel_name);
            continue;
        }

        // Create output file and XML writer
        let file = File::create(output_path).context("Failed to create output file")?;
        let buf_writer = BufWriter::new(file);
        let mut writer = Writer::new(buf_writer);

        // Process entries
        let mut ch = ch.clone();
        ch.set_entries(entries, start_date);

        // Write RSS
        ch.write_rss(&mut writer)?;

        tracing::info!("RSS feed written to {} with {} entries", output_path, ch.entries.len());
    }
    Ok(())
}

async fn scan_and_store(db: &Database, channel: &Channel, regex: &Regex) -> Result<()> {
    let path = Path::new(channel.file_path.as_str());

    let mut current_files = HashSet::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if regex.is_match(&file_name) {
                    current_files.insert(file_name);
                }
            }
        }
    }

    // Check against database
    let txn = db.begin_read()?;
    let table = txn.open_table(FILENAMES_TABLE)?;

    let mut new_files = Vec::new();

    for file in &current_files {
        if table.get(file.as_str())?.is_none() {
            new_files.push(file.clone());
        }
    }

    for file in &new_files {
        let fullpath = path.join(file.clone());
        match read_file_descriptor(fullpath.to_str().unwrap_or("invalid_path")) {
            Ok(records) => {
                let txn = db.begin_write()?;
                {
                    let mut table: redb::Table<'_, &str, Vec<u8>> = txn.open_table(FILEDESC_TABLE)?;
                    for file_desc in &records {
                        let serialized = serialize(&file_desc)?;
                        table.insert(file_desc.id.as_str(), serialized)?;
                    }
                }
                txn.commit()?;
                tracing::info!("Read {} descriptors from {}", records.len(), fullpath.to_str().unwrap_or("invalid_path"));

            },
            Err(e) => tracing::error!("Error reading file descriptor for {}: {}", fullpath.to_str().unwrap_or("invalid_path"), e),
        }
    }

    let txn = db.begin_write()?;
    {
        let mut table = txn.open_table(FILENAMES_TABLE)?;
        for file in &current_files {
            table.insert(file.as_str(), ())?;
        }
    }
    txn.commit()?;

    Ok(())
}

lazy_static! {
    static ref RE_ZSV_VIDEO_ID: Regex = Regex::new(r"^zsv(\d{6}[e]?)-(\d{1,3}[a-z]?)-(?:(\d{1,3}[a-z]?)-)?").expect("Invalid regex RE_ZSV_VIDEO_ID");
}

pub fn read_file_descriptor(path: &str) -> Result<Vec<FileDesc>> {
    // -----------------------------------------------------------------
    // 1. Open the .docx file (change the path if needed)
    // -----------------------------------------------------------------
    let path = std::path::Path::new(path);
    let mut file = std::fs::File::open(path)?;
    let mut buf = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut buf)?;
    let docx = docx_rs::read_docx(&buf)?;
    //let docx = Docx::read(file)?;

    // -----------------------------------------------------------------
    // 2. Find the first table (your list is the only table)
    // -----------------------------------------------------------------
    let table = docx
        .document
        .children
        .iter()
        .find_map(|child| match child {
            DocumentChild::Table(t) => Some(t),
            _ => None,
        })
        .ok_or_else(|| anyhow::anyhow!("No table found in the document"))?;

    // -----------------------------------------------------------------
    // 3. Skip the header row (順序 | 錄影內容 | 檔案數量)
    // -----------------------------------------------------------------
    let rows: Vec<_> = table.rows.iter().map(|child| match child {
        docx_rs::TableChild::TableRow(r) => r,
    }).collect();
    let data_rows = &rows[1..]; // everything after the header

    // -----------------------------------------------------------------
    // 4. Parse each row
    // -----------------------------------------------------------------
    let mut records = Vec::new();

    for row in data_rows {
        let cell_strings: Vec<String> = row
            .cells
            .iter()
            .map(|child| match child {
                docx_rs::TableRowChild::TableCell(c) => extract_text_from_cell(c),
            })
            .collect();
        let cells: Vec<&str> = cell_strings.iter().map(|s| s.trim()).collect();

        // Expected layout: [seq, name+desc, file_count]
        if cells.len() != 3 {
            eprintln!("Skipping malformed row: {:?}", cells);
            continue;
        }

        let seq = match cells[0].parse::<u32>() {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to parse seq '{}': {}", cells[0], e);
                continue
            }
        };
        let file_count = match cells[2].parse::<u32>() {
            Ok(fc) => fc,
            Err(e) => {
                tracing::error!("Failed to parse file_count '{}': {}", cells[2], e);
                continue
            }
        };

        // The second column contains BOTH the code name and the Chinese description,
        // separated by the transition from ASCII to Chinese characters.
        let full = cells[1];

        let first_chinese_byte = full.char_indices().find(|(_, c)| crate::models::formatter::is_chinese(*c)).map(|(i, _)| i);
        let (fname, chi_descr) = if let Some(pos) = first_chinese_byte {
            let name_part = full[..pos].trim();
            let desc_part = full[pos..].trim();
            (name_part.to_owned(), desc_part.to_owned())
        } else {
            (full.trim().to_owned(), String::new())
        };

        if let Some(caps) = RE_ZSV_VIDEO_ID.captures(&fname) {
            let prefix: &str = caps.get(0).expect("No match group 0").as_str();
            let id = format!("zsv{}-{}", &caps[1], &caps[2]);
            let mut eng_descr = fname.as_str().strip_prefix(prefix).unwrap_or(fname.as_str()).to_string();
            eng_descr = crate::models::formatter::format_eng_descr(&eng_descr);

            let file_desc = FileDesc {
                id: id.to_string(),
                seq,
                eng_descr: eng_descr.clone(),
                chi_descr: chi_descr.clone(),
                file_count,
            };

            records.push(file_desc);
        }
    }
    Ok(records)
}

// ---------------------------------------------------------------------
// Helper: pull plain text out of a table cell (handles paragraphs, runs…)
// ---------------------------------------------------------------------
fn extract_text_from_cell(cell: &TableCell) -> String {
    let mut text = String::new();
    for content in &cell.children {
        if let docx_rs::TableCellContent::Paragraph(p) = content {
            for run in &p.children {
                if let docx_rs::ParagraphChild::Run(r) = run {
                    for run_child in &r.children {
                        if let docx_rs::RunChild::Text(t) = run_child {
                            text.push_str(&t.text);
                        }
                    }
                }
            }
        }
    }
    text
}
