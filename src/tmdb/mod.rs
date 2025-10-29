use crate::tmdb::api::fetch_multi_page_with_retry;
use lazy_static::lazy_static;
use reqwest::Client;
use std::collections::HashMap;
use std::env;
use tokio::task::JoinSet;

mod api;
pub mod models;
pub const API_BASE_URL: &str = "https://api.themoviedb.org/3";
//todo 把 println 改为 log

static LANGUAGE: &str = "LANGUAGE";
// todo 添加代理
#[allow(dead_code)]
static PROXY: &str = "PROXY";
static TMDB_API_KEY: &str = "TMDB_API_KEY";
static QUERY: &str = "QUERY";
static INCLUDE_ADULT: &str = "INCLUDE_ADULT";

lazy_static! {
    pub static ref COMMON_QUERY: HashMap<String, String> = {
        let mut map = HashMap::new();
        if let Ok(language) = env::var(LANGUAGE) {
            map.insert(LANGUAGE.to_lowercase(), language);
        }
        if let Ok(api_key) = env::var(TMDB_API_KEY) {
            map.insert(TMDB_API_KEY.to_lowercase(), api_key);
        }
        if let Ok(include_adult) = env::var(INCLUDE_ADULT) {
            map.insert(INCLUDE_ADULT.to_lowercase(), include_adult);
        }
        map
    };
}

pub fn check_tmdb_id(tmdb_id: &u32) -> bool {
    if *tmdb_id != 0 {
        return true;
    }
    false
}

pub async fn query_tmdb_id(
    client: &Client,
    show_name: &str,
) -> Result<u32, Box<dyn std::error::Error>> {
    println!("\nSearching TMDB for '{show_name}'...");
    let mut queries = COMMON_QUERY.clone();
    queries.insert(QUERY.to_lowercase(), show_name.to_string());

    let mut all_results = fetch_multi_page_with_retry(client, API_BASE_URL, 0, &queries).await?;
    let mut join_set = JoinSet::new();
    if all_results.total_pages > 1 {
        for page_num in 2..=all_results.total_pages {
            let client_node = client.clone();
            let queries_clone = queries.clone();
            join_set.spawn(async move {
                fetch_multi_page_with_retry(&client_node, API_BASE_URL, page_num, &queries_clone)
                    .await
            });
        }
    }

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(page) => {
                all_results.results.extend(page?.results);
            }
            Err(e) => {
                eprintln!("Error fetching page: {}", e);
            }
        }
    }

    if all_results.results.is_empty() {
        return Err("No results found".into());
    }
    println!("  Found results {:?} .", all_results.results);
    Ok(all_results.results[0].id)
}

#[cfg(test)]
mod tests {}
