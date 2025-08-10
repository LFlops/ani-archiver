use regex::Regex;
use serde::{self, Deserialize, Deserializer};
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
pub async fn hash_files(
    path: &PathBuf,
) -> Result<(Vec<String>, Vec<PathBuf>), Box<dyn std::error::Error>> {
    let mut current_file_hashes = Vec::new();
    let mut video_files = Vec::new();
    for file_entry in fs::read_dir(path)? {
        let file_entry = file_entry?;
        if file_entry.path().is_file() {
            let hash = get_file_hash(&file_entry.path())?;
            current_file_hashes.push(hash);
            video_files.push(file_entry.path());
        }
    }
    current_file_hashes.sort();
    Ok((current_file_hashes, video_files))
}
pub fn get_file_hash(path: &Path) -> Result<String, io::Error> {
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

pub fn extract_episode_info(filename: &str) -> Option<(String, String)> {
    // todo 修改为根据字幕组匹配规则
    let re = Regex::new(r"[sS](\d{1,2})[eE](\d{1,2})|(\d{1,2})").unwrap();
    if let Some(caps) = re.captures(filename) {
        if let Some(s) = caps.get(1) {
            let s_num = s.as_str();
            let e_num = caps.get(2).unwrap().as_str();
            return Some((format!("{s_num:0>2}",), format!("{e_num:0>2}",)));
        } else if let Some(e) = caps.get(3) {
            return Some(("01".to_string(), format!("{:0>2}", e.as_str())));
        }
    }
    None
}

// Serde deserializer function to default null values to an empty string
pub fn deserialize_null_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_file_hash() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let hash = get_file_hash(file.path());
        assert!(hash.is_ok());
        let hash_str = hash.unwrap();
        assert_eq!(hash_str.len(), 64); // SHA256 produces 64 character hex string
    }

    #[test]
    fn test_extract_episode_info_with_season_and_episode() {
        // Test S01E02 format
        assert_eq!(
            extract_episode_info("show_S01E02.mp4"),
            Some(("01".to_string(), "02".to_string()))
        );

        // Test s3e12 format
        assert_eq!(
            extract_episode_info("show_s3e12.avi"),
            Some(("03".to_string(), "12".to_string()))
        );

        // Test with extra text
        assert_eq!(
            extract_episode_info("[MyGroup] Awesome Show S02E05 [1080p].mkv"),
            Some(("02".to_string(), "05".to_string()))
        );
    }

    #[test]
    fn test_extract_episode_info_with_only_episode() {
        // Test with just episode number
        assert_eq!(
            extract_episode_info("show_05.mp4"),
            Some(("01".to_string(), "05".to_string()))
        );

        // Test with multiple numbers - should match the first one as episode
        assert_eq!(
            extract_episode_info("show_12_extra.mp4"),
            Some(("01".to_string(), "12".to_string()))
        );
    }
}
