mod tmdb;
mod utils;

use crate::tmdb::models::TvShowDetails;
use dotenv::dotenv;
use reqwest::Client;
use std::env;
use std::fs;
use std::fs::hard_link;
use std::path::PathBuf;
use tmdb::models::ProcessedMarker;
use tmdb::nfo::create_tv_show_nfo;
use tmdb::{
    API_BASE_URL, choose_from_results, fetch_tv_show_details_with_client,
    search_tv_shows_with_client,
};
use utils::{extract_episode_info, get_file_hash};

//创建一个类型，用于存放从环境变量中获取的API密钥/Source/Dest 等
async fn local_env() -> (String, PathBuf, PathBuf) {
    let api_key = env::var("TMDB_API_KEY").expect("TMDB_API_KEY not set");
    let source: PathBuf = PathBuf::from(env::var("SOURCE").expect("SOURCE not set"));
    let dest = PathBuf::from(env::var("DEST").expect("DEST not set"));
    (api_key, source, dest)
}

async fn hash_files(
    path: &PathBuf,
) -> Result<(Vec<String>, Vec<PathBuf>), Box<dyn std::error::Error>> {
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
async fn check_processed(
    marker_file_path: &PathBuf,
    current_file_hashes: &Vec<String>,
    show_name: &str,
) -> Result<(u32, bool), Box<dyn std::error::Error>> {
    let mut tmdb_id = 0;
    let mut details_cached = false;

    if marker_file_path.exists() {
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
    }
    Ok((tmdb_id, details_cached))
}

async fn process_show(
    details_cached: bool,
    client: &Client,
    api_key: &str,
    tmdb_id: u32,
) -> Result<TvShowDetails, Box<dyn std::error::Error>> {
    let show_details = if details_cached {
        fetch_tv_show_details_with_client(&client, API_BASE_URL, &api_key, tmdb_id).await?
    } else {
        println!("Fetching details for TMDB ID {}...", tmdb_id);
        let details =
            fetch_tv_show_details_with_client(&client, API_BASE_URL, &api_key, tmdb_id).await?;
        details
    };
    Ok(show_details)
}
async fn check_tmdb_id(
    tmdb_id: &u32,
    client: &Client,
    show_name: &str,
    api_key: &str,
) -> Result<u32, Box<dyn std::error::Error>> {
    if *tmdb_id != 0 {
        return Ok(*tmdb_id);
    }

    println!("\nSearching TMDB for '{}'...", show_name);
    let search_results =
        search_tv_shows_with_client(client, API_BASE_URL, api_key, show_name).await?;
    if search_results.results.is_empty() {
        println!("No results found. Skipping.");
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("No TMDB results found for show: {}", show_name),
        )));
    }
    // todo 通过管道与其他逻辑节藕，避免阻塞整体。
    Ok(choose_from_results(search_results)?)
}
async fn write_marker(
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
async fn organize_files(
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
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    // let args = Args::parse();

    let (api_key, source, dest) = local_env().await;

    fs::create_dir_all(&dest)?;
    let client = Client::new();

    for entry in fs::read_dir(&source)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let show_name = path.file_name().unwrap().to_string_lossy().to_string();
            let dest_dir = dest.join(&show_name);

            let (current_file_hashes, video_files) = hash_files(&path).await?;

            let marker_file_path = dest_dir.join(".processed.json");
            let (mut tmdb_id, details_cached) =
                check_processed(&marker_file_path, &current_file_hashes, &show_name).await?;

            tmdb_id = match check_tmdb_id(&tmdb_id, &client, &show_name, &api_key).await {
                Ok(tmdb_id) => tmdb_id,
                Err(e) => {
                    println!("Error: {}", e);
                    continue;
                }
            };

            let show_details = process_show(details_cached, &client, &api_key, tmdb_id).await?;
            write_marker(&marker_file_path, &dest_dir, tmdb_id, current_file_hashes).await?;

            fs::create_dir_all(&dest_dir)?;
            let nfo_content = create_tv_show_nfo(&show_details);
            fs::write(dest_dir.join("tvshow.nfo"), nfo_content)?;
            organize_files(video_files, &dest_dir, &show_name).await?;
            println!("Successfully processed '{}'.", show_name);
        }
    }

    Ok(())
}
