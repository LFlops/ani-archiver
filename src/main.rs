mod file;
mod tmdb;
mod utils;

use crate::file::cache::check_processed;
use crate::file::{organize_files, write_marker};
use crate::tmdb::{check_tmdb_id, process_show};
use crate::utils::hash_files;
use dotenv::dotenv;
use file::nfo::create_tv_show_nfo;
use reqwest::Client;
use std::env;
use std::fs;
use std::path::PathBuf;

//创建一个类型，用于存放从环境变量中获取的API密钥/Source/Dest 等
async fn local_env() -> (String, PathBuf, PathBuf) {
    let api_key = env::var("TMDB_API_KEY").expect("TMDB_API_KEY not set");
    let source: PathBuf = PathBuf::from(env::var("SOURCE").expect("SOURCE not set"));
    let dest = PathBuf::from(env::var("DEST").expect("DEST not set"));
    (api_key, source, dest)
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
