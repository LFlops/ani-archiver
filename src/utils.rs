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