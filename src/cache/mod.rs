use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

const CACHE_FILE_NAME: &str = "cache.json";
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Cache {
    pub tmdb_id: u32,
    pub file_hashes: HashSet<String>,
}
impl Cache {
    pub fn new(tmdb_id: u32, file_hashes: HashSet<String>) -> Self {
        Self {
            tmdb_id,
            file_hashes,
        }
    }
    pub async fn write_cache(&self, dest_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        if !dest_dir.exists() {
            fs::create_dir_all(dest_dir)?;
        }
        let cache_content = serde_json::to_string(&self)?;
        let file_path = dest_dir.join(CACHE_FILE_NAME);
        fs::write(file_path, cache_content)?;
        Ok(())
    }

    pub async fn check_cache(
        &self,
        dest_dir: &PathBuf,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        if !dest_dir.exists() || !dest_dir.is_dir() {
            // 返回文件不存在的错误
            return Err(Box::new(std::io::Error::new(
                ErrorKind::NotADirectory,
                format!("{} does not exist", dest_dir),
            )));
        }
        let cache_file_path = dest_dir.join(CACHE_FILE_NAME);
        let local_cache: Cache = serde_json::from_reader(&fs::File::open(cache_file_path)?)?;
        Ok(self.tmdb_id == self.tmdb_id && local_cache.file_hashes == self.file_hashes)
    }
}
