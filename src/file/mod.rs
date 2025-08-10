use crate::file::models::ProcessedMarker;
use crate::utils::extract_episode_info;
use std::fs;
use std::fs::hard_link;
use std::path::PathBuf;

pub mod nfo;
pub mod models;
pub mod cache;

pub async fn write_marker(
    marker_file_path: &PathBuf,
    dest_dir: &PathBuf,
    tmdb_id: u32,
    current_file_hashes: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create marker file for persistence
    let marker = ProcessedMarker {
        tmdb_id,
        file_hashes: current_file_hashes,
    };
    let marker_content = serde_json::to_string(&marker)?;
    fs::create_dir_all(&dest_dir)?;
    fs::write(&marker_file_path, marker_content)?;
    Ok(())
}
pub async fn organize_files(
    video_files: Vec<PathBuf>,
    dest_dir: &PathBuf,
    show_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  Organizing files...");
    for video_file in video_files {
        if let Some(file_name) = video_file.file_name().and_then(|s| s.to_str()) {
            if let Some((s_num, e_num)) = extract_episode_info(file_name) {
                let new_file_name = format!(
                    "{} S{}E{}.{}",
                    show_name,
                    s_num,
                    e_num,
                    video_file.extension().unwrap().to_str().unwrap()
                );
                let hard_link_path = dest_dir.join(&new_file_name);

                // Check if file already exists to avoid errors on re-run
                if hard_link_path.exists() {
                    fs::remove_file(&hard_link_path)?;
                }

                // Create a hard link
                // todo 根据dest和source进行对比，优先使用硬链接，如果文件系统不同，则使用软链接，并给出提示。
                hard_link(&video_file, &hard_link_path)?;
                println!("    Created hard link: {}", hard_link_path.display());
            } else {
                println!(
                    "    Skipping '{}': Could not extract episode info.",
                    file_name
                );
            }
        }
    }
    Ok(())
}