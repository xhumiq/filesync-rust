use chrono::{NaiveDate, NaiveDateTime, Utc};

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
    /// Returns the first and last publication dates of entries.
    /// The entries vector is sorted by pub_date before extracting dates.
    /// Returns None if there are no entries.

    pub fn date_range(&self, start: NaiveDate, end: NaiveDate) -> Vec<MediaEntry> {
        self.entries.iter().filter(|entry| {
            let entry_date = entry.pub_date.date();
            entry_date >= start && entry_date <= end
        }).cloned().collect()
    }

    pub fn entries_for_date(&self, date: NaiveDate) -> Vec<MediaEntry> {
        self.entries.iter().filter(|e| e.pub_date.date() == date).cloned().collect()
    }


    pub fn entries_for_today(&self) -> Vec<MediaEntry> {
        let today = Utc::now().date_naive();
        self.entries_for_date(today)
    }
}


fn default_generator() -> String {
    "rss_writer".to_string()
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
    pub mime_type: String,
    pub size: u64,
    pub pub_date: NaiveDateTime,
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
            mime_type: String::new(),
            size: 0,
            pub_date: NaiveDate::from_ymd_opt(1970, 1, 1).expect("Invalid default date").and_hms_opt(0, 0, 0).expect("Invalid default time"),
            modified: std::time::UNIX_EPOCH,
        }
    }
}
