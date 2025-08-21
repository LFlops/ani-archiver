use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::{fs, io};

// todo 用 async io 替换 std io
const CACHE_FILE_NAME: &str = "cache.json";
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Cache {
    pub file_hashes: HashSet<String>,
}
impl Cache {
    pub fn new(file_hashes: HashSet<String>) -> Self {
        Self { file_hashes }
    }
    pub fn write_cache(&self, dest_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        if !dest_dir.exists() {
            fs::create_dir_all(dest_dir)?;
        }
        let cache_content = serde_json::to_string(&self)?;
        let file_path = dest_dir.join(CACHE_FILE_NAME);
        fs::write(file_path, cache_content)?;
        Ok(())
    }

    pub fn check_cache(&self, dest_dir: &PathBuf) -> Result<bool, Box<dyn std::error::Error>> {
        check_dir_path(dest_dir)?;
        let cache_file_path = dest_dir.join(CACHE_FILE_NAME);
        let local_cache: Cache = serde_json::from_reader(&fs::File::open(cache_file_path)?)?;
        Ok(local_cache.file_hashes == self.file_hashes)
    }

    pub async fn from_path(path: &PathBuf) -> Result<Cache, Box<dyn std::error::Error>> {
        check_dir_path(path)?;
        let mut local_hashes = HashSet::new();
        for file_entry in fs::read_dir(path)? {
            let hash = hash_one_file(&file_entry?.path())?;
            local_hashes.insert(hash);
        }

        Ok(Cache {
            file_hashes: local_hashes,
        })
    }
}

fn hash_one_file(path: &Path) -> Result<String, io::Error> {
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified()?;

    // 将SystemTime转换为duration since UNIX_EPOCH
    let duration = modified
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();

    let mut hasher = Sha256::new();
    // 使用文件的最后修改时间作为哈希输入，而不是文件内容
    hasher.update(format!("{}{}", duration.as_secs(), duration.subsec_nanos()).as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}

fn check_dir_path(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() || !path.is_dir() {
        return Err(Box::new(io::Error::new(
            ErrorKind::NotADirectory,
            format!("{} does not exist", path),
        )));
    }
    Ok(())
}
