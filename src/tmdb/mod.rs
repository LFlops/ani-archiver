use crate::tmdb::models::{FetchError, SearchMultiResult, SearchResponse, TvShowDetails};
use futures::future::ok;
use lazy_static::lazy_static;
use reqwest::{Client, StatusCode, Url};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::time::Duration;
use std::{env, io};
use thiserror::Error;
use tokio::task::JoinSet;

pub mod models;

static LANGUAGE: &str = "LANGUAGE";
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

pub const API_BASE_URL: &str = "https://api.themoviedb.org/3";
pub async fn process_show(
    details_cached: bool,
    client: &Client,
    api_key: &str,
    tmdb_id: u32,
) -> Result<TvShowDetails, Box<dyn std::error::Error>> {
    // todo 语言设定为中文
    let show_details = if details_cached {
        fetch_tv_show_details_with_client(client, API_BASE_URL, api_key, tmdb_id).await?
    } else {
        println!("Fetching details for TMDB ID {tmdb_id}...");
        fetch_tv_show_details_with_client(client, API_BASE_URL, api_key, tmdb_id).await?
    };
    Ok(show_details)
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
    // 同时搜索 movie 和 tv
    println!("\nSearching TMDB for '{show_name}'...");
    let mut queries = COMMON_QUERY.clone();
    queries.insert(QUERY.to_lowercase(), show_name.to_string());

    let mut all_results = fetch_page_with_retry(client, API_BASE_URL, 0, &queries).await?;
    let mut join_set = JoinSet::new();
    if all_results.total_pages > 1 {
        for page_num in 2..all_results.total_pages {
            let client_node = client.clone();
            let queries_clone = queries.clone();
            join_set.spawn(async move {
                fetch_page_with_retry(&client_node, API_BASE_URL, page_num, &queries_clone).await
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
    // todo 通过管道与其他逻辑节藕，避免阻塞整体。
    let mut stdout = io::stdout();
    let stdin = io::stdin();
    let mut buf_reader = io::BufReader::new(stdin);
    choose_from_results(&all_results, &mut buf_reader, &mut stdout)
}
// 可测试版本的函数，允许注入client和base_url
pub async fn fetch_tv_show_details_with_client(
    client: &Client,
    base_url: &str,
    api_key: &str,
    tv_show_id: u32,
) -> Result<TvShowDetails, reqwest::Error> {
    let url = format!("{base_url}/tv/{tv_show_id}");
    client
        .get(&url)
        .query(&[("api_key", api_key)])
        .send()
        .await?
        .json::<TvShowDetails>()
        .await
}

async fn fetch_page_with_retry(
    client: &Client,
    base_url: &str,
    page: u32,
    query: &HashMap<String, String>,
) -> Result<SearchResponse<SearchMultiResult>, FetchError> {
    let max_retries = 3;
    let mut base_delay = Duration::from_millis(500);
    let mut url = Url::parse(base_url).expect("Failed to parse base_url");
    url.path_segments_mut()
        .expect("cannot be base")
        .push("search")
        .push("multi");
    for attempt in 1..=max_retries {
        if attempt > 0 {
            println!(
                "第 {} 页第 {} 次重试，等待 {:?}...",
                page, attempt, base_delay
            );
            tokio::time::sleep(base_delay).await;
            base_delay *= 2;
        }
        println!("正在请求第 {} 页 (尝试次数 {})", page, attempt + 1);

        let response = client.get(url.clone()).query(query).send().await;
        match response {
            Ok(response) => {
                let status = response.status();
                if status == StatusCode::OK {
                    return Ok(response.json::<SearchResponse<SearchMultiResult>>().await?);
                } else if status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
                    eprintln!(
                        "Retrying page {} (attempt {}/{}): {}",
                        page,
                        attempt,
                        max_retries,
                        response.status()
                    )
                } else {
                    eprintln!("请求第 {} 页遇到不可恢复的错误， 状态码: {}", page, status);
                    return Err(FetchError::UnrecoverableStatus(status));
                }
            }
            Err(e) => {
                println!("Retrying (attempt {}/{}): {}", attempt, max_retries, e);
                continue;
            }
        }
    }
    Err(FetchError::MaxRetriesExceeded { max_retries })
}
// 可测试版本的函数，允许注入client和base_url

pub fn format_search_results(results: &SearchResponse<SearchMultiResult>) -> String {
    let mut output = String::new();
    output.push_str("Multiple results found, please choose one:\n");
    for (i, show) in results.results.iter().enumerate() {
        output.push_str(&format!(
            "{}. {} ({})\n",
            i + 1,
            show.name,
            show.first_air_date.as_deref().unwrap_or("????")
        ));
    }
    output
}
pub fn parse_choice(
    input: &str,
    results: &SearchResponse<SearchMultiResult>,
) -> Result<u32, &'static str> {
    match input.trim().parse::<usize>() {
        Ok(choice) if choice > 0 && choice <= results.results.len() => {
            // Valid choice, return the corresponding ID.
            Ok(results.results[choice - 1].id)
        }
        Ok(_) => {
            // Parsed as a number, but it's out of the valid range.
            Err("Choice is out of range.")
        }
        Err(_) => {
            // Failed to parse as a number.
            Err("Invalid input. Please enter a number.")
        }
    }
}
pub fn choose_from_results<R: BufRead, W: Write>(
    results: &SearchResponse<SearchMultiResult>,
    reader: &mut R,
    writer: &mut W,
) -> Result<u32, Box<dyn std::error::Error>> {
    // Write the formatted results to the provided writer.
    writeln!(writer, "{}", format_search_results(results))?;

    loop {
        // Write the prompt.
        write!(writer, "Enter number (1-{}): ", results.results.len())?;
        writer.flush()?; // Ensure the prompt is shown.

        let mut input = String::new();
        reader.read_line(&mut input)?;

        // Use our pure logic function to process the input.
        match parse_choice(&input, results) {
            Ok(id) => return Ok(id), // Valid choice, exit the loop and return.
            Err(msg) => writeln!(writer, "{msg}")?,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;
    use serde_json::json;

    #[tokio::test]
    async fn test_fetch_tv_show_details_success() {
        let tv_show_id = 1399;
        let api_key = "test_key";

        let mock_response = json!({
            "id": tv_show_id,
            "name": "Game of Thrones",
            "overview": "The best show ever",
            "genres": [{"name": "Action"}, {"name": "Adventure"}],
            "first_air_date": "2011-04-17",
            "vote_average": 8.4
        });

        let _m = mock("GET", format!("/tv/{tv_show_id}").as_str())
            .match_query(mockito::Matcher::UrlEncoded(
                "api_key".into(),
                api_key.into(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create();

        let client = Client::builder()
            .no_proxy()
            .build()
            .expect("Client build failed");
        let result =
            fetch_tv_show_details_with_client(&client, &mockito::server_url(), api_key, tv_show_id)
                .await;

        dbg!(&result);
        assert!(result.is_ok());
        if let Ok(show_details) = result {
            assert_eq!(show_details.id, tv_show_id);
            assert_eq!(show_details.name, "Game of Thrones");
            assert_eq!(show_details.vote_average, 8.4);
        }
    }

    #[tokio::test]
    async fn test_search_tv_shows_success() {
        let query = "Game of Thrones";
        let api_key = "test_key";

        let mock_response = json!({
            "page":1,
            "results": [
                {
                    "id": 1399,
                    "name": "Game of Thrones",
                    "first_air_date": "2011-04-17",
                    "overview": "The best show ever",
                    "adult": false,
                    "backdrop_path": "/zwHdg4RWuAsCPHwcxOrOMTDGcKi.jpg",
                    "genre_ids": [16, 18, 10759, 10765],
                    "origin_country": ["JP"],
                    "original_language": "ja",
                    "original_name": "ATRI -My Dear Moments-",
                    "popularity": 2.5202,
                    "poster_path": "/6bQKMlHwRmGnvIRxDahl1r8WJkE.jpg",
                    "vote_average": 7.938,
                    "vote_count": 16
                }
            ],
            "total_pages": 1,
            "total_results": 1
        });

        let _m = mock("GET", "/search/tv")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("api_key".into(), api_key.into()),
                mockito::Matcher::UrlEncoded("query".into(), query.into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create();

        let client = Client::builder()
            .no_proxy()
            .build()
            .expect("Client build failed");
        let result =
            fetch_page_with_retry(&client, &mockito::server_url(), 0, &HashMap::new()).await;

        assert!(result.is_ok());
        if let Ok(search_response) = result {
            assert_eq!(search_response.results.len(), 1);
            assert_eq!(search_response.results[0].id, 1399);
            assert_eq!(search_response.results[0].name, "Game of Thrones");
            assert_eq!(
                search_response.results[0].overview,
                Some("The best show ever".to_string())
            );
        }
    }

    #[tokio::test]
    async fn test_fetch_tv_show_details_error() {
        let tv_show_id = 999999; // 不存在的ID
        let api_key = "test_key";

        let _m = mock("GET", format!("/tv/{tv_show_id}").as_str())
            .match_query(mockito::Matcher::UrlEncoded(
                "api_key".into(),
                api_key.into(),
            ))
            .with_status(404)
            .create();

        let client = Client::builder()
            .no_proxy()
            .build()
            .expect("Client build failed");
        let result =
            fetch_tv_show_details_with_client(&client, &mockito::server_url(), api_key, tv_show_id)
                .await;

        assert!(result.is_err());
    }

    #[test]
    fn test_api_base_url() {
        assert_eq!(API_BASE_URL, "https://api.themoviedb.org/3");
    }

    #[tokio::test]
    async fn test_fetch_tv_show_details_url_format() {
        let tv_show_id = 12345u32;
        let url = format!("{API_BASE_URL}/tv/{tv_show_id}");
        assert_eq!(url, "https://api.themoviedb.org/3/tv/12345");
    }

    #[tokio::test]
    async fn test_search_tv_shows_url_format() {
        let url = format!("{API_BASE_URL}/search/tv",);
        assert_eq!(url, "https://api.themoviedb.org/3/search/tv");
    }
}
