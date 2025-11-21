use chrono::prelude::*;
use chrono::{DateTime, Utc, Local, NaiveDate, Duration};
use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;
use crate::models::files::MediaEntry;

lazy_static! {
    static ref RE_MULTIPLE_SPACES: Regex = Regex::new(r" +").expect("Invalid regex RE_MULTIPLE_SPACES");
    static ref RE_ALC588WMM: Regex = Regex::new(r"ALC 588 WMM").expect("Invalid regex RE_ALC588WMM");
    static ref RE_ALC588: Regex = Regex::new(r"ALC 588").expect("Invalid regex RE_ALC588");
    static ref RE_DATE_DIGITS: Regex = Regex::new(r"\b(\d{6})\b").expect("Invalid regex RE_DATE_DIGITS");
}

pub fn clean_pub_date(entries: Vec<MediaEntry>) -> Vec<MediaEntry> {
    let mut groups: HashMap<NaiveDate, Vec<MediaEntry>> = HashMap::new();
    for entry in entries {
        groups.entry(entry.pub_date).or_insert(Vec::new()).push(entry);
    }
    let mut result = Vec::new();
    for (pub_date_date, mut group) in groups {
        group.sort_by_key(|e| e.modified);
        if let Some(first) = group.first() {
            let first_modified = first.modified;
            let cutoff = first_modified + Duration::hours(1).to_std().expect("Invalid duration");
            let base_entry = group.iter().rev().find(|e| e.modified <= cutoff).unwrap_or(first);
            let mut base_time = base_entry.modified;
            let base_date = DateTime::<Utc>::from(base_time).date_naive();
            if base_date.day() != pub_date_date.day() {
                let adjusted_date = pub_date_date + Duration::days(1);
                let adjusted_datetime = adjusted_date.and_hms_opt(23, 59, 59).expect("Invalid time 23:59:59");
                base_time = DateTime::<Utc>::from_naive_utc_and_offset(adjusted_datetime, Utc).into();
            }
            base_time = base_time - std::time::Duration::from_secs((group.len() + 1) as u64);
            // if base_time has a time of day at 0 zero hours and zero minuites and zero seconds then add 5 minutes to it
            let dt = DateTime::<Local>::from(base_time);
            if dt.hour() == 0 && dt.minute() == 0 && dt.second() == 0 {
                base_time = (dt + chrono::Duration::minutes(5)).into();
            }
            for mut entry in group {
                entry.pub_date = DateTime::<Utc>::from(base_time).date_naive();
                result.push(entry);
            }
        }
    }
    result
}

// ---------------------------------------------------------------------
// Helper: check if a character is Chinese
// ---------------------------------------------------------------------
pub fn is_chinese(c: char) -> bool {
    // CJK Unified Ideographs
    ('\u{4e00}' <= c && c <= '\u{9fff}') ||
    // CJK Extension A
    ('\u{3400}' <= c && c <= '\u{4dbf}') ||
    // CJK Extension B
    ('\u{20000}' <= c && c <= '\u{2a6df}') ||
    // CJK Extension C
    ('\u{2a700}' <= c && c <= '\u{2b73f}')
}

// ---------------------------------------------------------------------
// Helper: extract Chinese characters from a string
// ---------------------------------------------------------------------
pub fn extract_chinese(s: &str) -> String {
    s.chars()
        .filter(|c| is_chinese(*c))
        .collect()
}

// ---------------------------------------------------------------------
// Helper: format name by inserting spaces before capital letters followed by lowercase, between letters and digits, replacing dashes with spaces, and formatting 6-digit dates
// ---------------------------------------------------------------------
pub fn format_eng_descr(s: &str) -> String {
    // Now apply spacing
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();
    let mut prev_is_digit = false;
    let mut prev_is_letter = false;
    for i in 0..chars.len() {
        let c = chars[i];
        let is_digit = c.is_ascii_digit();
        let is_letter = c.is_alphabetic();
        if i > 0 && chars[i-1].is_lowercase() && c.is_uppercase() {
            result.push(' ');
        }
        if i > 0 && ((prev_is_letter && is_digit) || (prev_is_digit && is_letter)) {
            result.push(' ');
        }
        if c == '-' {
            result.push(' ');
        } else if c == ',' {
            result.push(',');
            result.push(' ');
        } else if c == '&' {
            result.push(' ');
            result.push('&');
            result.push(' ');
        } else {
            result.push(c);
        }
        prev_is_digit = is_digit;
        prev_is_letter = is_letter;
    }
    // Replace multiple spaces with single space
    let result = RE_MULTIPLE_SPACES.replace_all(&result, " ").to_string();
    let result = RE_ALC588WMM.replace_all(&result, "ALC/588/WMM").to_string();
    let result = RE_ALC588.replace_all(&result, "ALC/588").to_string();

    // First, replace 6-digit dates
    RE_DATE_DIGITS.replace_all(&result, |caps: &regex::Captures| {
        let digits = &caps[1];
        if let Ok(date) = NaiveDate::parse_from_str(digits, "%y%m%d") {
            date.format("%Y-%m-%d").to_string()
        } else {
            digits.to_string()
        }
    }).to_string()

}

pub fn format_event_date(ed: &str) -> String {
    if ed.len() == 6 {
        format!(" 20{}-{}-{}", &ed[0..2], &ed[2..4], &ed[4..6])
    } else {
        String::new()
    }
}

pub fn normalize_location(loc: &str) -> String {
    match loc {
        "MH" => "MtHermon".to_string(),
        "KL" => "Kuala Lumper".to_string(),
        "KK" => "Kota Kinabalu".to_string(),
        "CL" => "Canaan Land".to_string(),
        "IL" => "Isaac Land".to_string(),
        "DL" => "Dawnlight".to_string(),
        "AU" => "Australia".to_string(),
        "US" => "United States".to_string(),
        "CA" => "Canada".to_string(),
        "LA" => "Los Angeles".to_string(),
        "Joseph" => "Joseph Land".to_string(),
        "Olive" => "MtOlive".to_string(),
        "Carmel" => "MtCarmel".to_string(),
        _ => loc.to_string(),
    }
}

pub fn parseMediaType(filename: &str) -> String {
    let path = std::path::Path::new(filename);
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    if let Some(mime) = MIME_TYPE_MAP.get(ext.as_str()) {
        if mime.starts_with("video/") {
            "video".to_string()
        } else if mime.starts_with("audio/") {
            "audio".to_string()
        } else if mime.starts_with("image/") {
            "image".to_string()
        } else if mime.starts_with("image/") {
            "image".to_string()
        } else if path.ends_with(".zip") || path.ends_with(".rar") || path.ends_with(".7z") || path.ends_with(".tar") || path.ends_with(".gz") {
            "archive".to_string()
        }else{
            "blob".to_string()
        }
    } else {
        "unknown".to_string()
    }
}

lazy_static! {
    pub static ref MIME_TYPE_MAP: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        // Video formats
        map.insert("mp4", "video/mp4");
        map.insert("avi", "video/x-msvideo");
        map.insert("wmv", "video/x-ms-wmv");
        map.insert("mkv", "video/x-matroska");
        map.insert("mov", "video/quicktime");
        map.insert("flv", "video/x-flv");
        map.insert("webm", "video/webm");
        map.insert("m4v", "video/x-m4v");
        map.insert("3gp", "video/3gpp");
        map.insert("mpg", "video/mpeg");
        map.insert("mpeg", "video/mpeg");
        // Audio formats
        map.insert("mp3", "audio/mpeg");
        map.insert("wav", "audio/wav");
        map.insert("flac", "audio/flac");
        map.insert("aac", "audio/aac");
        map.insert("ogg", "audio/ogg");
        map.insert("wma", "audio/x-ms-wma");
        map.insert("m4a", "audio/mp4");
        map.insert("opus", "audio/opus");
        // Image formats
        map.insert("jpg", "image/jpeg");
        map.insert("jpeg", "image/jpeg");
        map.insert("png", "image/png");
        map.insert("gif", "image/gif");
        map.insert("bmp", "image/bmp");
        map.insert("tiff", "image/tiff");
        map.insert("tif", "image/tiff");
        map.insert("svg", "image/svg+xml");
        map.insert("webp", "image/webp");
        // Document formats
        map.insert("pdf", "application/pdf");
        map.insert("doc", "application/msword");
        map.insert("docx", "application/vnd.openxmlformats-officedocument.wordprocessingml.document");
        map.insert("xls", "application/vnd.ms-excel");
        map.insert("xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
        map.insert("ppt", "application/vnd.ms-powerpoint");
        map.insert("pptx", "application/vnd.openxmlformats-officedocument.presentationml.presentation");
        map.insert("txt", "text/plain");
        map.insert("rtf", "application/rtf");
        // Archive formats
        map.insert("zip", "application/zip");
        map.insert("rar", "application/x-rar-compressed");
        map.insert("7z", "application/x-7z-compressed");
        map.insert("tar", "application/x-tar");
        map.insert("gz", "application/gzip");
        // Other
        map.insert("json", "application/json");
        map.insert("xml", "application/xml");
        map.insert("html", "text/html");
        map.insert("css", "text/css");
        map.insert("js", "application/javascript");
        map
    };
}
