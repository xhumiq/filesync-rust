use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Record {
    pub seq: u32,
    pub name: String,
    pub description: String,
    pub file_count: u32,
}
