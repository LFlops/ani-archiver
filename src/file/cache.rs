use crate::file::models::ProcessedMarker;
use std::path::PathBuf;
use std::fs;

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

    let marker_content = fs::read_to_string(&marker_file_path)?;
    let marker: ProcessedMarker = serde_json::from_str(&marker_content)?;

    let mut stored_hashes = marker.file_hashes;
    stored_hashes.sort();

    if stored_hashes == *current_file_hashes {
        tmdb_id = marker.tmdb_id;
        details_cached = true;
        println!(
            "\n'{}' is up-to-date. Using cached TMDB ID {}.",
            show_name, tmdb_id
        );
    }
    Ok((tmdb_id, details_cached))
}
