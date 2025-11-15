use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FileDesc {
    pub id: String,
    pub seq: u32,
    pub eng_descr: String,
    pub chi_descr: String,
    pub file_count: u32,
}
