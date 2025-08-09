mod tmdb;
mod utils;

use clap::Parser;
use dotenv::dotenv;
use tmdb::models::{Args, ProcessedMarker};
use tmdb::nfo::create_tv_show_nfo;
use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use tmdb::{choose_from_results, fetch_tv_show_details, search_tv_shows};
use utils::{extract_episode_info, get_file_hash};
//创建一个类型，用于存放从环境变量中获取的API密钥/Source/Dest 等
async fn local_env() -> (String, PathBuf, PathBuf) {
    let api_key = env::var("TMDB_API_KEY").expect("TMDB_API_KEY not set");
    let source:PathBuf = PathBuf::from(env::var("SOURCE").expect("SOURCE not set"));
    let dest = PathBuf::from(env::var("DEST").expect("DEST not set"));
    (api_key, source, dest)
}

async fn hash_files(path: &PathBuf) -> Result<(Vec<String>, Vec<PathBuf>), Box<dyn std::error::Error>> {
    let mut current_file_hashes = Vec::new();
    let mut video_files = Vec::new();
    for file_entry in fs::read_dir(&path)? {
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
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    // let args = Args::parse();

    let (api_key, source, dest) = local_env().await;


    fs::create_dir_all(&dest)?;

    for entry in fs::read_dir(&source)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let show_name = path.file_name().unwrap().to_string_lossy().to_string();
            let dest_dir = dest.join(&show_name);
            let marker_file_path = dest_dir.join(".processed.json");

            let (current_file_hashes,video_files) = hash_files(&path).await?;

            let mut tmdb_id = 0;
            let mut details_cached = false;

            if  marker_file_path.exists() {
                let marker_content = fs::read_to_string(&marker_file_path)?;
                let marker: ProcessedMarker = serde_json::from_str(&marker_content)?;

                let mut stored_hashes = marker.file_hashes;
                stored_hashes.sort();

                if stored_hashes == current_file_hashes {
                    tmdb_id = marker.tmdb_id;
                    details_cached = true;
                    println!(
                        "\n'{}' is up-to-date. Using cached TMDB ID {}.",
                        show_name, tmdb_id
                    );
                }
            }

            if tmdb_id == 0 {
                println!("\nSearching TMDB for '{}'...", show_name);
                let search_results = search_tv_shows(&api_key, &show_name).await?;
                if search_results.results.is_empty() {
                    println!("  No results found. Skipping.");
                    continue;
                }
                tmdb_id = choose_from_results(search_results)?;
            }

            let show_details = if details_cached {
                fetch_tv_show_details(&api_key, tmdb_id).await?
            } else {
                println!("Fetching details for TMDB ID {}...", tmdb_id);
                let details = fetch_tv_show_details(&api_key, tmdb_id).await?;

                // Create marker file for persistence
                let marker = ProcessedMarker {
                    tmdb_id,
                    file_hashes: current_file_hashes,
                };
                let marker_content = serde_json::to_string(&marker)?;
                fs::create_dir_all(&dest_dir)?;
                fs::write(&marker_file_path, marker_content)?;

                details
            };

            fs::create_dir_all(&dest_dir)?;
            let nfo_content = create_tv_show_nfo(&show_details);
            fs::write(dest_dir.join("tvshow.nfo"), nfo_content)?;

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

                        // Create a symbolic link
                        symlink(&video_file, &hard_link_path)?;
                        println!("    Created hard link: {}", hard_link_path.display());
                    } else {
                        println!(
                            "    Skipping '{}': Could not extract episode info.",
                            file_name
                        );
                    }
                }
            }

            println!("Successfully processed '{}'.", show_name);
        }
    }

    Ok(())
}
