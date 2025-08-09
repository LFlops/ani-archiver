use regex::Regex;
use serde::{self, Deserialize, Deserializer};
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::Path;

pub fn get_file_hash(path: &Path) -> Result<String, io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn extract_episode_info(filename: &str) -> Option<(String, String)> {
    let re = Regex::new(r"[sS](\d{1,2})[eE](\d{1,2})|(\d{1,2})").unwrap();
    if let Some(caps) = re.captures(filename) {
        if let Some(s) = caps.get(1) {
            let s_num = s.as_str();
            let e_num = caps.get(2).unwrap().as_str();
            return Some((format!("{:0>2}", s_num), format!("{:0>2}", e_num)));
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
