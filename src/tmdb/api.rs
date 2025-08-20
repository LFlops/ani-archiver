use std::collections::HashMap;
use std::time::Duration;
use reqwest::{Client, StatusCode, Url};
use crate::tmdb::models::{FetchError, SearchMultiResult, SearchResponse};

pub(super) async fn fetch_multi_page_with_retry(
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