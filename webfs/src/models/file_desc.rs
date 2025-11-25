use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileDesc {
    pub id: String,
    pub seq: u32,
    pub eng_descr: String,
    pub chi_descr: String,
    pub file_count: u32,
}
