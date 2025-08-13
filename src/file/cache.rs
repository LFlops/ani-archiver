use crate::file::models::ProcessedMarker;
use std::path::{Path, PathBuf};
use std::fs;

const VIDEO_EXTENSIONS: [&str; 4] = [".mkv", ".mp4", ".avi", ".m4v"];
const SUBTITLE_EXTENSIONS: [&str; 2] = [".srt", ".ass"];

pub async fn check_file_extensions(file_path: &Path) -> bool {
    let file_extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    VIDEO_EXTENSIONS.contains(&file_extension) || SUBTITLE_EXTENSIONS.contains(&file_extension)
}

pub async fn check_filename_by_regex(file_path: &Path) -> bool {
    
    let file_name = file_path.file_name().unwrap().to_string_lossy();
    let regex = regex::Regex::new(r"^[a-zA-Z0-9\s\-_]+$").unwrap();
    regex.is_match(&file_name)
}
pub async fn check_processed(
    marker_file_path: &PathBuf,
    current_file_hashes: &Vec<String>,
    show_name: &str,
) -> Result<(u32, bool), Box<dyn std::error::Error>> {
    let mut tmdb_id = 0;
    let mut details_cached = false;
    if !marker_file_path.exists() {
        return Ok((tmdb_id, details_cached));
    }

    let marker_content = fs::read_to_string(marker_file_path)?;
    let marker: ProcessedMarker = serde_json::from_str(&marker_content)?;

    let mut stored_hashes = marker.file_hashes;
    stored_hashes.sort();

    if stored_hashes == *current_file_hashes {
        tmdb_id = marker.tmdb_id;
        details_cached = true;
        println!(
            "\n'{show_name}' is up-to-date. Using cached TMDB ID {tmdb_id}."
        );
    }
    Ok((tmdb_id, details_cached))
}
