mod cache;
mod common;
mod local_file;
mod tmdb;

use crate::cache::Cache;
use crate::tmdb::{check_tmdb_id, query_tmdb_id};
use dotenv::dotenv;
use log::warn;
use reqwest::{Client, Proxy};
use std::env;
use std::fs;
use std::path::PathBuf;

//创建一个类型，用于存放从环境变量中获取的API密钥/Source/Dest 等
async fn local_env() -> Result<(String, PathBuf, PathBuf, Option<Proxy>), Box<dyn std::error::Error>>
{
    let api_key = env::var("TMDB_API_KEY").expect("TMDB_API_KEY not set");
    let source: PathBuf = PathBuf::from(env::var("SOURCE").expect("SOURCE not set"));
    let dest = PathBuf::from(env::var("DEST").expect("DEST not set"));
    let proxy = match env::var("PROXY") {
        Ok(proxy_url) => Some(Proxy::https(&proxy_url)?),
        Err(_) => None,
    };
    Ok((api_key, source, dest, proxy))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let (_api_key, source, dest, proxy) = local_env().await?;
    fs::create_dir_all(&dest)?;

    let mut client_builder = Client::builder().timeout(std::time::Duration::from_secs(30));
    if let Some(proxy) = proxy {
        client_builder = client_builder.proxy(proxy);
    }
    let client = client_builder.build()?;

    for entry in fs::read_dir(&source)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let show_name = path.file_name().unwrap().to_string_lossy().to_string();

            let tmdb_id = match query_tmdb_id(&client, &show_name).await {
                Ok(tmdb_id) => tmdb_id,
                Err(e) => {
                    println!("Error fetching TMDB ID for '{show_name}': {e}");
                    continue;
                }
            };
            if !check_tmdb_id(&tmdb_id) {
                warn!("Invalid TMDB ID for '{show_name}': {tmdb_id}");
            }

            let dest_dir = dest.join(&show_name);
            if !dest_dir.is_dir() {
                fs::create_dir_all(&dest_dir)?;
            }
            let cache = Cache::from_path(&path).await?;
            if cache.check_cache(&dest_dir)? {
                continue;
            }
            cache.write_cache(&dest_dir)?;
            // todo
            let local_file = local_file::LocalFile::<PathBuf>::from_env();
            local_file.organize_files(&show_name).await?;
            println!("Successfully processed '{show_name}'.");
        }
    }

    Ok(())
}
