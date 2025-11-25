use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
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
use crate::storage::Storage;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;

pub struct MonitorConfig{
    pub config: Config,
    pub db_path: String,
    pub video_descr_file_pattern: String,
    pub rss_days: i32,
    pub rss_output_path: String,
    pub video_list_path: String,    
}

pub async fn start_file_monitor(config: &MonitorConfig, storage: Arc<Mutex<Storage>>, cache: Arc<Mutex<HashMap<String, (Channel, chrono::DateTime<chrono::Utc>)>>>) -> Result<(), Box<dyn std::error::Error>> {
    let pattern = config.video_descr_file_pattern.as_str();
    let regex = Regex::new(pattern)?;

    let scan_path = config.video_list_path.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5)); // Poll every 5 seconds
        loop {
            interval.tick().await;
            tracing::info!("Scanning files... {}", scan_path);
            if let Err(e) = scan_and_store(&storage, scan_path.as_str(), &regex).await {
                tracing::error!("Error scanning files: {}", e);
            }
        }
    });
    let mut rss_days = config.rss_days;
    if rss_days >= 0{
        if rss_days == 0{
            rss_days = 7;
        }
        let start_date = Utc::now().date_naive() - chrono::Duration::days(rss_days as i64);
        let rss_channels: Vec<(String, Channel)> = config.config.channels.values().flat_map(|m| m.iter().map(|(k,v)| (k.clone(), v.clone()))).collect();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(5)); // Poll every 5 seconds
            loop {
                interval.tick().await;
                if let Err(e) = write_rss(&rss_channels, start_date, &cache){
                    tracing::error!("Error writing RSS: {}", e);
                }
            }
        });
    }else{
        tracing::warn!("RSS Refresh Skipped - RSS_DAYS not set");
    }
    Ok(())
}

fn write_rss(channels_to_process: &[(String, Channel)], start_date: NaiveDate, cache: &std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, (Channel, chrono::DateTime<chrono::Utc>)>>>) -> Result<()> {
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
        ch.set_entries(entries);

        // Cache the channel
        {
            let mut cache = cache.lock().unwrap();
            cache.insert(channel_name.to_string(), (ch.clone(), Utc::now()));
        }

        // Write RSS
        ch.write_rss(&mut writer, Some(start_date))?;

        tracing::info!("RSS feed written to {} with {} entries", output_path, ch.entries.len());
    }
    Ok(())
}

async fn scan_and_store(storage: &Arc<Mutex<Storage>>, scan_path: &str, regex: &Regex) -> Result<()> {
    let path = Path::new(scan_path);
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

    let mut new_files = Vec::new();
    let storage = storage.lock().unwrap();
    for file in &current_files {
        if !storage.filename_exists(file)? {
            new_files.push(file.clone());
        }
    }

    new_files.sort();

    for file in &new_files {
        let fullpath = path.join(file.clone());
        match read_file_descriptor(fullpath.to_str().unwrap_or("invalid_path")) {
            Ok(records) => {
                storage.insert_file_descs(&records)?;
                tracing::info!("Read {} descriptors from {}", records.len(), fullpath.to_str().unwrap_or("invalid_path"));
            },
            Err(e) => tracing::error!("Error reading file descriptor for {}: {}", fullpath.to_str().unwrap_or("invalid_path"), e),
        }
    }

    storage.insert_filenames(&current_files.into_iter().collect::<Vec<_>>())?;

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
