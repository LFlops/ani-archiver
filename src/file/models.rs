use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessedMarker {
    pub tmdb_id: u32,
    pub file_hashes: Vec<String>,
}
