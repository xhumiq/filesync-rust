use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use chrono::prelude::*;
use regex::Regex;
use lazy_static::lazy_static;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use chrono::{DateTime, Utc, Local, NaiveDate, Duration};
use anyhow::Result;
use std::collections::HashMap;
use tracing;

// The Config structure contains a hashmap of another hashmap of channels.
// The first key is language - "en", "zh"
// The second key is channel name
#[derive(serde::Deserialize, Clone)]
pub struct Config {
    pub channels: HashMap<String, HashMap<String, Channel>>,
    #[serde(default)]
    pub paths: HashMap<String, HashMap<String, Channel>>,
    pub default: ChannelDefaults,
}

impl Config {
    pub fn get_folder_info(&mut self, lang: &str, path: &str) -> Result<Channel> {
        let channel = self.paths.get(lang)
            .and_then(|lang_map| lang_map.get(path));

            let channel = if let Some(channel) = channel {
                channel.clone()
            } else {
                // Check if base_path + path exists as dir
                let full_path = PathBuf::from(&path);
                if !full_path.exists() || !full_path.is_dir() {
                    // Log available paths for the language
                    if let Some(lang_map) = self.paths.get(lang) {
                        let available_paths: Vec<&String> = lang_map.keys().collect();
                        tracing::trace!("Path '{}' not found. Available paths for language '{}': {:?}", path, lang, available_paths);
                    } else {
                        tracing::trace!("Path '{}' not found. No paths configured for language '{}'", path, lang);
                    }
                    return Err(anyhow::anyhow!("Path '{}' not found", path));
                }
                // Initialize new Channel
                let path_components: Vec<&str> = path.split('/').collect();
                let title = if path_components.len() >= 2 {
                    let last_two = &path_components[path_components.len() - 2..];
                    last_two.join(" ")
                } else {
                    path.to_string()
                };
                let relative = path.strip_prefix(&self.default.base_file_path).unwrap_or(&path).trim_start_matches('/');
                let mut channel = Channel {
                    name: title.replace(" ", "_").to_lowercase(),
                    title: format!("GJCC {}", title),
                    description: format!("GJCC Content {}", title),
                    media_link: format!("https://{}.{}{}{}", self.default.server_name, self.default.domain, self.default.base_media_url, relative),
                    server_name: self.default.server_name.clone(),
                    category: self.default.category.clone(),
                    author: self.default.author.clone(),
                    generator: self.default.generator.clone(),
                    file_path: path.to_string(),
                    ..Default::default()
                };
                channel.link = channel.media_link.clone();
                channel.output_path = format!("{}/{}.rss", self.default.base_output_path.clone(), channel.name.to_lowercase());
                channel
            };
        Ok(channel)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChannelDefaults {
    #[serde(default = "default_domain")]
    pub domain: String,
    #[serde(default = "default_base_media_url")]
    pub base_media_url: String,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default = "default_author")]
    pub author: String,
    #[serde(default = "default_generator")]
    pub generator: String,
    pub server_name: String,
    #[serde(default = "default_base_file_path")]
    pub base_file_path: String,
    #[serde(default = "default_base_output_path")]
    pub base_output_path: String,
}

impl Default for ChannelDefaults {
    fn default() -> Self {
        Self {
            domain: "ziongjcc.org".to_string(),
            base_media_url: "/".to_string(),
            category: "Christian".to_string(),
            author: "GJCC".to_string(),
            generator: "rss_writer".to_string(),
            server_name: "MUST BE SET".to_string(),
            base_file_path: "/srv/media".to_string(),
            base_output_path: "/srv/rss".to_string(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Channel {
    #[serde(default)]
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub link: String,
    pub media_link: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: String,
    pub language: String,
    #[serde(default)]
    pub author: String,
    #[serde(default = "default_generator")]
    pub generator: String,
    #[serde(default)]
    pub server_name: String,
    pub file_path: String,
    #[serde(default)]
    pub filter_extension: String,
    #[serde(default)]
    pub output_path: String,
    #[serde(default)]
    pub image: String,
    #[serde(default)]
    pub image_path: String,
    #[serde(default)]
	pub entries: Vec<MediaEntry>,
}

impl Default for Channel {
    fn default() -> Self {
        Self {
            name: String::new(),
            title: String::new(),
            link: String::new(),
            media_link: String::new(),
            description: String::new(),
            category: String::new(),
            language: "en-us".to_string(),
            author: String::new(),
            generator: "rss_writer".to_string(),
            server_name: "localhost".to_string(),
            file_path: String::new(),
            filter_extension: String::new(),
            output_path: String::new(),
            image: String::new(),
            image_path: String::new(),
            entries: Vec::new(),
        }
    }
}

impl Channel {
    pub fn read_config(path: &str) -> Result<Config> {
        let file = File::open(path)?;
        let mut config: Config = serde_yaml::from_reader(file)?;
        // Populate paths from channels
        for (lang, channels) in &config.channels {
            let mut path_map = HashMap::new();
            for (_name, channel) in channels {
                path_map.insert(channel.file_path.clone(), channel.clone());
            }
            config.paths.insert(lang.clone(), path_map);
        }

        // Fill in default values for config.default
        if config.default.domain.is_empty() {
            config.default.domain = default_domain();
        }
        if config.default.base_media_url.is_empty() {
            config.default.base_media_url = default_base_media_url();
        }
        if config.default.category.is_empty() {
            config.default.category = default_category();
        }
        if config.default.author.is_empty() {
            config.default.author = default_author();
        }
        if config.default.generator.is_empty() {
            config.default.generator = default_generator();
        }
        if config.default.base_file_path.is_empty() {
            config.default.base_file_path = default_base_file_path();
        }
        if config.default.base_output_path.is_empty() {
            config.default.base_output_path = default_base_output_path();
        }

        // Fill in default values for channels
        for (_lang, channels) in &mut config.channels {
            for (_name, channel) in channels {
                if channel.name.is_empty() {
                    channel.name = _name.clone();
                }
                if channel.title.is_empty() {
                    channel.title = format!("GJCC {}", _name);
                }
                if channel.description.is_empty() {
                    channel.title = format!("GJCC Content {}", _name);
                }
                if channel.server_name.is_empty() {
                    channel.server_name = config.default.server_name.clone();
                }
                if channel.media_link.is_empty() {
                    let relative = channel.file_path.strip_prefix(&config.default.base_file_path).unwrap_or(&channel.file_path).trim_start_matches('/');
                    channel.media_link = format!("https://{}.{}{}{}", channel.server_name, config.default.domain, config.default.base_media_url, relative);
                }
                if channel.link.is_empty() {
                    channel.link = channel.media_link.clone();
                }
                if channel.category.is_empty() {
                    channel.category = config.default.category.clone();
                }
                if channel.author.is_empty() {
                    channel.author = config.default.author.clone();
                }
                if channel.generator.is_empty() {
                    channel.generator = config.default.generator.clone();
                }
                if channel.output_path.is_empty() {
                    channel.output_path = format!("{}/{}.rss", config.default.base_output_path.clone(), _name.to_lowercase());
                }
            }
        }

        Ok(config)
    }

    pub fn read_dir(channel: &Channel) -> std::io::Result<Vec<MediaEntry>> {
        let start = std::time::Instant::now();
        let files: Vec<_> = Self::read_dir_sequential(channel)?;
        let duration = start.elapsed();

        tracing::info!("Time to read directory: {:?} Total files: {}", duration, files.len());
        Ok(files)
    }

    // Sequential version (FASTER for ≤35k files)
    fn read_dir_sequential(channel: &Channel) -> std::io::Result<Vec<MediaEntry>> {
        let path = Path::new(&channel.file_path);
        let files: Vec<MediaEntry> = fs::read_dir(path)?
            .flatten()
            .filter_map(|entry| MediaEntry::from_entry(entry, channel).ok())
            .collect();
        Ok(files)
    }
    pub fn set_entries(&mut self, entries: Vec<MediaEntry>, start_date: Option<NaiveDate>) {
        let mut files: Vec<MediaEntry> = if self.filter_extension.is_empty() || self.filter_extension == "*" {
            entries
        } else {
            entries.into_iter().filter(|e| e.file_name.ends_with(&self.filter_extension)).collect()
        };
        if let Some(start_date) = start_date {
            files = files.into_iter().filter(|entry| {
                entry.pub_date >= start_date
            }).collect();
        }
        files = Self::clean_pub_date(files);
        files.sort_by(|a, b| {
            let mut date_cmp = b.file_date_stamp.cmp(&a.file_date_stamp);
            if date_cmp == std::cmp::Ordering::Equal {
                date_cmp = a.event.cmp(&b.event);
                if date_cmp == std::cmp::Ordering::Equal {
                    a.index.cmp(&b.index)
                } else {
                    date_cmp
                }
            } else {
                date_cmp
            }
        });
        // Adjust pub_date to add days for same date entries
        let mut groups: HashMap<String, Vec<&mut MediaEntry>> = HashMap::new();
        for entry in &mut files {
            groups.entry(entry.file_date_stamp.clone()).or_insert(Vec::new()).push(entry);
        }
        for (_date_stamp, mut group) in groups {
            if group.len() > 1 {
                let base_time = group[0].pub_date;
                let len = group.len() as i64;
                for (i, entry) in group.iter_mut().enumerate() {
                    entry.pub_date = base_time + chrono::Duration::days(len - 1 - i as i64);
                }
            }
        }
        self.entries = files;
    }

    fn clean_pub_date(entries: Vec<MediaEntry>) -> Vec<MediaEntry> {
        let mut groups: HashMap<NaiveDate, Vec<MediaEntry>> = HashMap::new();
        for entry in entries {
            groups.entry(entry.pub_date).or_insert(Vec::new()).push(entry);
        }
        let mut result = Vec::new();
        for (pub_date_date, mut group) in groups {
            group.sort_by_key(|e| e.modified);
            if let Some(first) = group.first() {
                let first_modified = first.modified;
                let cutoff = first_modified + Duration::hours(1).to_std().unwrap();
                let base_entry = group.iter().rev().find(|e| e.modified <= cutoff).unwrap_or(first);
                let mut base_time = base_entry.modified;
                let base_date = DateTime::<Utc>::from(base_time).date_naive();
                if base_date.day() != pub_date_date.day() {
                    let adjusted_date = pub_date_date + Duration::days(1);
                    let adjusted_datetime = adjusted_date.and_hms_opt(23, 59, 59).unwrap();
                    base_time = DateTime::<Utc>::from_naive_utc_and_offset(adjusted_datetime, Utc).into();
                }
                base_time = base_time - std::time::Duration::from_secs((group.len() + 1) as u64);
                // if base_time has a time of day at 0 zero hours and zero minuites and zero seconds then add 5 minutes to it
                let dt = DateTime::<Local>::from(base_time);
                if dt.hour() == 0 && dt.minute() == 0 && dt.second() == 0 {
                    base_time = (dt + chrono::Duration::minutes(5)).into();
                }
                for mut entry in group {
                    // Convert base_time back to NaiveDate for pub_date
                    entry.pub_date = DateTime::<Utc>::from(base_time).date_naive();
                    result.push(entry);
                }
            }
        }
        result
    }

    pub fn write_rss<W: std::io::Write>(&mut self, writer: &mut Writer<W>) -> Result<()> {

        // Start RSS root element
        let mut rss_start = BytesStart::new("rss");
        rss_start.push_attribute(("version", "2.0"));
        rss_start.push_attribute(("xmlns:itunes", "http://www.itunes.com/dtds/podcast-1.0.dtd"));
        writer.write_event(Event::Start(rss_start))?;

        // Start channel element
        writer.write_event(Event::Start(BytesStart::new("channel")))?;

        // Add channel metadata
        write_element(writer, "title", &self.title)?;
        write_element(writer, "link", &self.link)?;
        write_element(writer, "description", &self.description)?;
        write_element(writer, "language", &self.language)?;
        write_element(writer, "generator", "rssWriter v0.3.5-15")?;
        let now = Local::now();
        let last_build_date = now.to_rfc2822();
        write_element(writer, "lastBuildDate", &last_build_date)?;
        let mut category = BytesStart::new("category");
        category.push_attribute(("text", "Christianity"));
        writer.write_event(Event::Empty(category))?;

        // iTunes channel elements
        write_element(writer, "itunes:author", "GJCC")?;
        write_element(writer, "itunes:explicit", "no")?;
        let mut category = BytesStart::new("itunes:category");
        category.push_attribute(("text", "Christianity"));
        writer.write_event(Event::Empty(category))?;
        let now = Local::now();
        let subtitle = format!("{} Pub: {}", &self.title, now.format("%a %b %d %H:%M:%S %Z %Y"));
        write_element(writer, "itunes:subtitle", &subtitle)?;

        // Add items for each entry
        for entry in &self.entries {
            entry.write_rss_item(writer, &self.media_link)?;
        }

        // End channel and RSS
        writer.write_event(Event::End(BytesEnd::new("channel")))?;
        writer.write_event(Event::End(BytesEnd::new("rss")))?;

        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MediaEntry {
	pub guid: String,
	pub title: String,
	pub link: String,
	pub description: String,
    pub content_type: String,
    pub file_name: String,
    pub file_date_stamp: String,
    pub day_night: String,
    pub event: String,
    pub event_code: String,
    pub index: String,
    pub event_desc: String,
    pub location: String,
    pub event_date_stamp: String,
    pub media_type: String,
    pub size: u64,
    pub pub_date: NaiveDate,
    pub modified: std::time::SystemTime,
}

impl Default for MediaEntry {
    fn default() -> Self {
        Self {
            guid: String::new(),
            title: String::new(),
            link: String::new(),
            description: String::new(),
            content_type: String::new(),
            file_name: String::new(),
            file_date_stamp: String::new(),
            day_night: String::new(),
            event: String::new(),
            event_code: String::new(),
            index: String::new(),
            event_desc: String::new(),
            location: String::new(),
            event_date_stamp: String::new(),
            media_type: String::new(),
            size: 0,
            pub_date: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            modified: std::time::UNIX_EPOCH,
        }
    }
}

impl MediaEntry {
    pub fn new(guid: String, title: String, link: String, description: String, pub_date: NaiveDate, size: u64, modified: std::time::SystemTime) -> MediaEntry {
        MediaEntry {
            guid,
            title,
            link,
            description,
            pub_date,
            size,
            modified,
            ..Default::default()
        }
    }
    pub fn from_entry(entry: std::fs::DirEntry, channel: &Channel) -> std::io::Result<Self> {
        let metadata = entry.metadata()?;
        if !metadata.is_file() {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "not a file"));
        }

        let path_str = entry.path().to_string_lossy().to_string();
        let mut fi = parseFileName(&path_str);
        fi.file_name = entry.file_name().to_string_lossy().to_string();
        fi.size = metadata.len();
        fi.modified = metadata.modified()?;
        // Set pub_date based on file_date_stamp if valid, otherwise use modified time
        fi.pub_date = if let Ok(date) = NaiveDate::parse_from_str(&fi.file_date_stamp, "%y%m%d") {
            date
        } else {
            let modified_dt = DateTime::<Utc>::from(metadata.modified()?);
            let date = modified_dt.date_naive();
            fi.file_date_stamp = date.format("%y%m%d").to_string();
            date
        };
        fi.guid = format!("{}/{}", channel.server_name, fi.file_name);
        fi.fill_rss_fields(channel);
        Ok(fi)
    }

    fn format_event_date(ed: &str) -> String {
        if ed.len() == 6 {
            format!(" 20{}-{}-{}", &ed[0..2], &ed[2..4], &ed[4..6])
        } else {
            String::new()
        }
    }

    fn normalize_location(loc: &str) -> String {
        match loc {
            "MH" => "MtHermon".to_string(),
            "Olive" => "MtOlive".to_string(),
            "Carmel" => "MtCarmel".to_string(),
            _ => loc.to_string(),
        }
    }

    fn construct_title(&self) -> String {
        let mut evt = self.event.clone();
        if !self.day_night.is_empty() {
            evt = format!("{}{}", self.day_night, evt);
        }
        let idx = if self.index.is_empty() { String::new() } else { format!("-{}", self.index) };
        let cd = contentDesc(&self.event_code, &self.event_desc);
        let cd = if cd.is_empty() { String::new() } else { format!(" {}", cd) };
        let loc = Self::normalize_location(&self.location);
        let loc = if loc.is_empty() { String::new() } else { format!(" {}", loc) };
        let ed = Self::format_event_date(&self.event_date_stamp);
        format!("{}{}{}{}{}", evt, idx, cd, loc, ed)
    }

    fn construct_description(&self) -> String {
        let mut evt = self.event.clone();
        if !self.day_night.is_empty() {
            evt = format!("{}{}", self.day_night, evt);
        }
        let idx = if self.index.is_empty() { String::new() } else { format!("-{}", self.index) };
        let evn = if self.day_night == "e" { " Evening" } else { "" };
        let loc = Self::normalize_location(&self.location);
        let loc = if loc.is_empty() { String::new() } else { format!(" {}", loc) };
        let sub = if self.event_desc.is_empty() { String::new() } else { format!(" {}", self.event_desc.replace("M.V.", "Music Video")) };
        let ed = Self::format_event_date(&self.event_date_stamp);
        format!("{}{}{}{}{}{}", evt, idx, evn, loc, sub, ed)
    }

    fn format_released_date(&self) -> String {
        Self::format_event_date(&self.file_date_stamp).trim_start().to_string()
    }

    pub fn fill_rss_fields(&mut self, channel: &Channel) {
        let channel_title = &channel.title;
        if self.title.is_empty(){
            self.title = format!("{} {}", self.format_released_date(), self.construct_title());
        }
        if self.description.is_empty(){
            self.description = format!("{} {} {}", channel_title, self.format_released_date(), self.construct_description());
        }
        if self.link.is_empty(){
            self.link = format!("{}/{}", channel.media_link.trim_end_matches('/'), self.file_name);
        }
        //self.pub_date = self.modified;
    }

    pub fn write_rss_item<W: std::io::Write>(&self, writer: &mut Writer<W>, media_link: &str) -> Result<()> {
        let url = format!("{}/{}", media_link.trim_end_matches('/'), self.file_name);
        let datetime = self.pub_date.and_hms_opt(0, 0, 0).unwrap();
        let pub_date: String = DateTime::<Utc>::from_naive_utc_and_offset(datetime, Utc).to_rfc3339();

        // Start item
        writer.write_event(Event::Start(BytesStart::new("item")))?;

        // Title
        write_element(writer, "title", &self.title)?;

        // Description
        write_element(writer, "description", &self.description)?;

        // Enclosure
        let ext = self.file_name.rsplit('.').next().unwrap_or("").to_lowercase();
        let mime_type = MIME_TYPE_MAP.get(ext.as_str()).copied().unwrap_or("application/octet-stream");
        let mut enclosure = BytesStart::new("enclosure");
        enclosure.push_attribute(("url", url.as_str()));
        enclosure.push_attribute(("length", self.size.to_string().as_str()));
        enclosure.push_attribute(("type", mime_type));
        writer.write_event(Event::Empty(enclosure))?;

        // PubDate
        write_element(writer, "pubDate", &pub_date)?;

        // GUID
        write_element(writer, "guid", &self.guid)?;

        // iTunes Author
        write_element(writer, "itunes:author", "GJCC")?;

        // End item
        writer.write_event(Event::End(BytesEnd::new("item")))?;

        Ok(())
    }
}

fn write_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    tag: &str,
    content: &str,
) -> Result<()> {
    writer.write_event(Event::Start(BytesStart::new(tag)))?;
    writer.write_event(Event::Text(BytesText::new(content)))?;
    writer.write_event(Event::End(BytesEnd::new(tag)))?;
    Ok(())
}


fn parseMediaType(filename: &str) -> String {
    let path = std::path::Path::new(filename);
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    if let Some(mime) = MIME_TYPE_MAP.get(ext.as_str()) {
        if mime.starts_with("video/") {
            "video".to_string()
        } else if mime.starts_with("audio/") {
            "audio".to_string()
        } else if mime.starts_with("image/") {
            "image".to_string()
        } else {
            "blob".to_string()
        }
    } else {
        "unknown".to_string()
    }
}

fn parseFileName(filename: &str) -> MediaEntry {
    let base = std::path::Path::new(filename).file_name().unwrap_or_default().to_string_lossy();
    let mut fi = MediaEntry {
        media_type: parseMediaType(filename),
        file_name: base.to_string(),
        ..Default::default()
    };
    if let Some(caps) = RE_ZSV_PATTERN.captures(&base) {
        fi.content_type = "zsf".to_string();
        fi.file_date_stamp = caps.get(1).map_or("", |m| m.as_str()).to_string();
        fi.day_night = caps.get(2).map_or("", |m| m.as_str()).to_string();
        fi.event = caps.get(3).map_or("", |m| m.as_str()).to_string();
        if fi.event == "List" {
            fi.event_desc = fi.event.clone();
            fi.event = "".to_string();
        }
        if fi.event.len() > 1 {
            fi.event_code = fi.event.chars().last().unwrap().to_string();
        }
        fi.index = caps.get(4).map_or("", |m| m.as_str()).to_string();
        fi.event_desc = caps.get(5).map_or("", |m| m.as_str()).trim_matches('-').to_string();
        if !fi.event_desc.is_empty() {
            if let Some(caps_desc) = RE_ZSV_DESC_DATED.captures(&fi.event_desc) {
                fi.location = caps_desc.get(1).map_or("", |m| m.as_str()).to_string();
                fi.event_date_stamp = caps_desc.get(2).map_or("", |m| m.as_str()).to_string();
                fi.day_night = caps_desc.get(3).map_or("", |m| m.as_str()).to_string();
                fi.event_desc = caps_desc.get(4).map_or("", |m| m.as_str()).to_string();
            } else if let Some(caps_desc) = RE_ZSV_DESC_PATTERN.captures(&fi.event_desc) {
                fi.location = caps_desc.get(1).map_or("", |m| m.as_str()).to_string();
                fi.event_date_stamp = caps_desc.get(2).map_or("", |m| m.as_str()).to_string();
                fi.event_desc = caps_desc.get(3).map_or("", |m| m.as_str()).to_string();
            }
        }
        return fi;
    }

    if let Some(caps) = RE_ANY_FULL_PATTERN.captures(&base) {
        fi.content_type = caps.get(1).map_or("", |m| m.as_str()).to_string();
        fi.file_date_stamp = caps.get(2).map_or("", |m| m.as_str()).to_string();
        fi.day_night = caps.get(3).map_or("", |m| m.as_str()).to_string();
        fi.event = caps.get(4).map_or("", |m| m.as_str()).to_string();
        if fi.event == "List" {
            fi.event_desc = fi.event.clone();
            fi.event = "".to_string();
        }
        if fi.event.len() > 1 {
            fi.event_code = fi.event.chars().last().unwrap().to_string();
        }
        fi.event_desc = caps.get(5).map_or("", |m| m.as_str()).trim_matches('-').to_string();
        if !fi.event_desc.is_empty() {
            if let Some(caps_desc) = RE_ZSV_DESC_DATED.captures(&fi.event_desc) {
                fi.location = caps_desc.get(1).map_or("", |m| m.as_str()).to_string();
                fi.event_date_stamp = caps_desc.get(2).map_or("", |m| m.as_str()).to_string();
                fi.day_night = caps_desc.get(3).map_or("", |m| m.as_str()).to_string();
                fi.event_desc = caps_desc.get(4).map_or("", |m| m.as_str()).to_string();
            } else if let Some(caps_desc) = RE_ZSV_DESC_PATTERN.captures(&fi.event_desc) {
                fi.location = caps_desc.get(1).map_or("", |m| m.as_str()).to_string();
                fi.event_date_stamp = caps_desc.get(2).map_or("", |m| m.as_str()).to_string();
                fi.event_desc = caps_desc.get(3).map_or("", |m| m.as_str()).to_string();
            }
        }
        return fi;
    }

    if let Some(caps) = RE_ZS_PATTERN.captures(&base) {
        fi.content_type = "zs".to_string();
        fi.file_date_stamp = caps.get(1).map_or("", |m| m.as_str()).to_string();
        fi.day_night = caps.get(2).map_or("", |m| m.as_str()).to_string();
        fi.event = caps.get(4).map_or("", |m| m.as_str()).to_string();
        if fi.event.starts_with("e") && fi.event.len() > 2 {
            fi.day_night = "e".to_string();
            fi.event = fi.event[1..].to_string();
        }
        if fi.event.len() > 1 {
            fi.event_code = fi.event.chars().last().unwrap().to_string();
        }
        fi.event_desc = caps.get(5).map_or("", |m| m.as_str()).trim_matches('-').to_string();
        if fi.event_desc.is_empty() {
            fi.event_desc = contentDesc(&fi.event_code, "");
        } else if let Some(caps_desc) = RE_ZSV_DESC_PATTERN.captures(&fi.event_desc) {
            fi.location = caps_desc.get(1).map_or("", |m| m.as_str()).to_string();
            fi.event_date_stamp = caps_desc.get(2).map_or("", |m| m.as_str()).to_string();
            fi.event_desc = caps_desc.get(3).map_or("", |m| m.as_str()).to_string();
        }
        return fi;
    }

    if let Some(caps) = RE_HYMN_PATTERN.captures(&base) {
        fi.content_type = "zs".to_string();
        fi.file_date_stamp = caps.get(1).map_or("", |m| m.as_str()).to_string();
        fi.event = caps.get(2).map_or("", |m| m.as_str()).to_string();
        if fi.event.len() > 1 {
            fi.event_code = fi.event.chars().next().unwrap().to_string();
            fi.index = fi.event[1..].to_string();
        }
        fi.event_desc = caps.get(3).map_or("", |m| m.as_str()).trim_matches('-').to_string();
        if !fi.event_desc.is_empty() {
            fi.event_desc = format!("{}_{}", contentDesc(&fi.event_code, ""), fi.event_desc);
        }
    }
    fi.title = Path::new(&fi.file_name).file_stem().unwrap_or_default().to_string_lossy().to_string();
    fi
}

fn contentDesc(contentType: &str, event_desc: &str) -> String {
    match contentType {
        "r" => "Report".to_string(),
        "v" => "Video".to_string(),
        "c" => {
            event_desc.to_string()
        },
        "n" => "News".to_string(),
        "z" => "Life".to_string(),
        "a" => "Prayer".to_string(),
        "s" => "Hymn".to_string(),
        "h" => "Grandpa".to_string(),
        "" => "".to_string(),
        _ => format!("Type {}", contentType.to_uppercase()),
    }
}

fn default_language() -> String {
    "en-us".to_string()
}

fn default_generator() -> String {
    "rss_writer".to_string()
}

fn default_domain() -> String {
    "ziongjcc.org".to_string()
}

fn default_base_media_url() -> String {
    "/".to_string()
}

fn default_category() -> String {
    "Christian".to_string()
}

fn default_author() -> String {
    "GJCC".to_string()
}

fn default_base_file_path() -> String {
    "/home/mchu/Videos/ZSF".to_string()
}

fn default_base_output_path() -> String {
    "/ntc/tmp".to_string()
}

const PARALLEL_THRESHOLD: usize = 35000;

lazy_static! {
    static ref RE_ZSV_PATTERN: Regex = Regex::new(r"^zsv(\d{6})(e?)-(\d{1,2}[a-z]|\w+)(?:-(\d{1,2}z?)(?:-([^(.]+))?)?").unwrap();
    static ref RE_ANY_FULL_PATTERN: Regex = Regex::new(r"^([A-Za-z]+)(\d{8})(e?)-(\d{1,2}[a-z]|\w+)(?:-(.+))?.mp4").unwrap();
    static ref RE_ZSV_DESC_PATTERN: Regex = Regex::new(r"^([\w][\w\d]+)(?:[-]?(\d{6}|\d{2}\.\d{2}\.\d{4}))?-([^(.]+)").unwrap();
    static ref RE_ZSV_DESC_DATED: Regex = Regex::new(r"(.*?)(?:[-_])?(\d{6}|\d{2}\.\d{2}\.\d{4})(e)?(?:-([^(.]+))?").unwrap();
    static ref RE_ZS_PATTERN: Regex = Regex::new(r"^zs(\d{6})(e?)(?:-?([a-z]{1,3}))?-(e?\d{1,2}[a-z]z?)(?:-?([^(.]+))?").unwrap();
    static ref RE_HYMN_PATTERN: Regex = Regex::new(r"^zs(\d{6})-(s\d{1,2})-h(\d{4})(?:-?([^(.]+))?").unwrap();
    static ref MIME_TYPE_MAP: HashMap<&'static str, &'static str> = {
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
