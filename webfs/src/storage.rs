use anyhow::Result;
use redb::{Database, TableDefinition};
use std::fs;
use std::path::Path;
use bincode;
use crate::models::file_desc::FileDesc;
use crate::models::files::MediaEntry;

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
}
