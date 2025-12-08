use anyhow::Result;
use redb::{Database, TableDefinition};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use bincode;
use chrono::{Utc, DateTime};
use crate::models::file_desc::FileDesc;
use crate::models::files::{Channel, MediaEntry};
use std::sync::{Arc, Mutex};

const CHANNEL_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("channel");
const FILENAMES_TABLE: TableDefinition<&str, ()> = TableDefinition::new("filenames");
const FILEDESC_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("filedesc");

pub struct Storage {
    db: Database,
}

impl Storage {
    pub fn new(db_path: &str) -> Result<Self> {
        // Create the database directory if it doesn't exist
        if let Some(parent) = Path::new(db_path).parent() {
            fs::create_dir_all(parent).map_err(|e| {
                tracing::error!("Failed to create directory {}: {}", parent.display(), e);
                e
            })?;
        }

        let db = Database::create(db_path).map_err(|e| {
            tracing::error!("Failed to create database at {}: {}", db_path, e);
            e
        })?;

        // Ensure the tables exist
        {
            let txn = db.begin_write().map_err(|e| {
                tracing::error!("Failed to begin write transaction: {}", e);
                e
            })?;
            txn.open_table(FILENAMES_TABLE).map_err(|e| {
                tracing::error!("Failed to open filenames table: {}", e);
                e
            })?;
            txn.open_table(FILEDESC_TABLE).map_err(|e| {
                tracing::error!("Failed to open filedesc table: {}", e);
                e
            })?;
            txn.open_table(CHANNEL_TABLE).map_err(|e| {
                tracing::error!("Failed to open filedesc table: {}", e);
                e
            })?;
            txn.commit().map_err(|e| {
                tracing::error!("Failed to commit transaction: {}", e);
                e
            })?;
        }

        Ok(Storage { db })
    }

    pub fn insert_file_desc(&self, file_desc: &FileDesc) -> Result<()> {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(FILEDESC_TABLE)?;
            let serialized = bincode::serialize(file_desc)?;
            table.insert(file_desc.id.as_str(), serialized)?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn insert_file_descs(&self, file_descs: &[FileDesc]) -> Result<()> {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(FILEDESC_TABLE)?;
            for file_desc in file_descs {
                let serialized = bincode::serialize(file_desc)?;
                table.insert(file_desc.id.as_str(), serialized)?;
            }
        }
        txn.commit()?;
        Ok(())
    }

    pub fn filename_exists(&self, filename: &str) -> Result<bool> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(FILENAMES_TABLE)?;
        Ok(table.get(filename)?.is_some())
    }

    pub fn insert_filename(&self, filename: &str) -> Result<()> {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(FILENAMES_TABLE)?;
            table.insert(filename, ())?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn insert_filenames(&self, filenames: &[String]) -> Result<()> {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(FILENAMES_TABLE)?;
            for filename in filenames {
                table.insert(filename.as_str(), ())?;
            }
        }
        txn.commit()?;
        Ok(())
    }

    pub fn insert_channel(&self, channel: &Channel) -> Result<()> {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(CHANNEL_TABLE)?;
            let serialized = bincode::serialize(channel)?;
            let id = format!("{}/{}", channel.copy_lang, channel.name);
            table.insert(id.as_str(), serialized)?;
        }
        txn.commit()?;
        Ok(())
    }

    pub fn get_file_desc(&self, id: &str) -> Result<Option<FileDesc>> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(FILEDESC_TABLE)?;
        let serialized = table.get(id)?.map(|v| bincode::deserialize(v.value().as_slice()).unwrap());
        Ok(serialized)
    }

    pub fn get_batch_file_desc(&self, ids: &[&str]) -> Result<Vec<FileDesc>> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(FILEDESC_TABLE)?;
        let mut entities = Vec::new();
        for id in ids {
            if let Some(v) = table.get(id)? {
                let desc: FileDesc = bincode::deserialize(v.value().as_slice()).unwrap();
                entities.push(desc);
            }
        }
        Ok(entities)
    }

    pub fn channel_descriptions(&self, ch: Channel, cache: Arc<Mutex<HashMap<String, (Channel, chrono::DateTime<chrono::Utc>)>>>) -> Result<(Channel, bool)> {
        let cached_ch_option = {
            let _cache: std::sync::MutexGuard<'_, HashMap<String, (Channel, chrono::DateTime<Utc>)>> = cache.lock().unwrap();
            _cache.get(&ch.cache_id()).cloned()
        };
        let filled_ch = {
            self.fill_descriptions(&ch, &cached_ch_option)
        };
        match filled_ch {
            Ok(filled_ch) => {
                // Check if channel has changed
                let changed = if let Some((ref cached_ch, _)) = cached_ch_option {
                    let current_info: Vec<_> = filled_ch.entries.iter().map(|e| (&e.file_name, &e.description, &e.pub_date)).collect();
                    let cached_info: Vec<_> = cached_ch.entries.iter().map(|e| (&e.file_name, &e.description, &e.pub_date)).collect();
                    current_info != cached_info
                } else {
                    true
                };

                if changed {
                    let mut cache = cache.lock().unwrap();
                    cache.insert(ch.cache_id().to_string(), (filled_ch.clone(), Utc::now()));
                }
                Ok((filled_ch, changed))
            }
            Err(e) => {
                tracing::error!("Error filling descriptions for {}: {}", &ch.cache_id(), e);
                Err(e)
            }
        }
    }

    pub fn fill_descriptions(&self, channel: &Channel, cached_ch: &Option<(Channel, DateTime<Utc>)>) -> Result<Channel> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(FILEDESC_TABLE)?;
        let mut entries = Vec::new();
        let mut entryMap: HashMap<String, MediaEntry> = HashMap::new();
        if let Some((ref cached_ch, _)) = cached_ch {
            for entry in &cached_ch.entries {
                entryMap.insert(entry.normalized_event_id("zsv"), entry.clone());
            }
        }
        for entry in &channel.entries {
            let mut entry = entry.clone();
            let key = entry.normalized_event_id("zsv");
            if let Some(desc) = table.get(key.as_str())?.map(|v| bincode::deserialize::<FileDesc>(v.value().as_slice()).unwrap()) {
                if channel.copy_lang.starts_with("zh") {
                    if !desc.chi_descr.is_empty() { 
                        let mut evt = crate::models::formatter::normalize_code(&entry.event).to_string();
                        if !entry.index.is_empty() { evt = format!("{}-{}", evt, entry.index); }
                        if !evt.is_empty() { evt = format!(" ({})", evt).to_string(); }
                        entry.description = format!("{}{}", desc.chi_descr, evt);
                    }
                }else if channel.copy_lang.starts_with("en"){
                    if !desc.eng_descr.is_empty() { 
                        let mut evt = crate::models::formatter::normalize_code(&entry.event).to_string();
                        if !entry.index.is_empty() { evt = format!("{}-{}", evt, entry.index); }
                        if !evt.is_empty() { evt = format!(" ({})", evt).to_string(); }
                        entry.description = format!("{}{}", desc.eng_descr, evt);
                    };
                }
            }else if channel.copy_lang == "zh" {
                if let Some(cached_entry) = entryMap.get(&key) {
                    entry.description = cached_entry.description.clone();
                }
            }
            entries.push(entry);
        }
        let mut channel = channel.clone();
        channel.set_entries(entries);
        Ok(channel)
    }
}
