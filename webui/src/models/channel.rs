use chrono::{NaiveDate, Utc};

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
    pub fn first_and_last_dates(&mut self) -> Option<(NaiveDate, NaiveDate)> {
        if self.entries.is_empty() {
            return None;
        }

        // Sort entries by pub_date
        self.entries.sort_by(|a, b| a.pub_date.cmp(&b.pub_date));

        let first_date = self.entries.first()?.pub_date;
        let last_date = self.entries.last()?.pub_date;

        Some((first_date, last_date))
    }

    pub fn date_range(&self, start: NaiveDate, end: NaiveDate) -> Vec<MediaEntry> {
        self.entries.iter().filter(|entry| {
            entry.pub_date >= start && entry.pub_date <= end
        }).cloned().collect()
    }

    pub fn entries_for_date(&self, date: NaiveDate) -> Vec<MediaEntry> {
        self.entries.iter().filter(|e| e.pub_date == date).cloned().collect()
    }

    pub fn past_3_days(&self) -> Vec<MediaEntry> {
        let today = Utc::now().date_naive();
        let start = today - chrono::Duration::days(2);
        let end = today + chrono::Duration::days(1);
        self.date_range(start, end)
    }

    pub fn entries_for_today(&self) -> Vec<MediaEntry> {
        let today = Utc::now().date_naive();
        self.entries_for_date(today)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::UNIX_EPOCH;

    fn create_test_entry(pub_date: NaiveDate) -> MediaEntry {
        MediaEntry {
            guid: "test".to_string(),
            title: "Test".to_string(),
            link: "test".to_string(),
            description: "test".to_string(),
            content_type: "test".to_string(),
            file_name: "test".to_string(),
            file_date_stamp: "test".to_string(),
            day_night: "test".to_string(),
            event: "test".to_string(),
            event_code: "test".to_string(),
            index: "test".to_string(),
            event_desc: "test".to_string(),
            location: "test".to_string(),
            event_date_stamp: "test".to_string(),
            media_type: "test".to_string(),
            size: 0,
            pub_date,
            modified: UNIX_EPOCH,
        }
    }

    #[test]
    fn test_first_and_last_dates_empty() {
        let mut channel = Channel::default();
        assert_eq!(channel.first_and_last_dates(), None);
    }

    #[test]
    fn test_first_and_last_dates_single_entry() {
        let mut channel = Channel::default();
        let test_date = NaiveDate::from_ymd_opt(2024, 1, 1).expect("Invalid test date");
        channel.entries.push(create_test_entry(test_date));

        let result = channel.first_and_last_dates();
        assert_eq!(result, Some((test_date, test_date)));
    }

    #[test]
    fn test_first_and_last_dates_multiple_entries() {
        let mut channel = Channel::default();

        // Create entries with different dates (out of order)
        let date1 = NaiveDate::from_ymd_opt(2024, 1, 2).expect("Invalid test date"); // middle
        let date2 = NaiveDate::from_ymd_opt(2024, 1, 4).expect("Invalid test date"); // latest
        let date3 = NaiveDate::from_ymd_opt(2024, 1, 1).expect("Invalid test date"); // earliest

        channel.entries.push(create_test_entry(date2)); // latest
        channel.entries.push(create_test_entry(date1)); // middle
        channel.entries.push(create_test_entry(date3)); // earliest

        let result = channel.first_and_last_dates();
        assert_eq!(result, Some((date3, date2))); // Should be sorted: date3 (Jan 1) first, date2 (Jan 4) last
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
            pub_date: NaiveDate::from_ymd_opt(1970, 1, 1).expect("Invalid default date"),
            modified: std::time::UNIX_EPOCH,
        }
    }
}
