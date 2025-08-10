use crate::tmdb::models::{SearchResponse, TvShowDetails};
use reqwest::Client;

pub mod models;
pub mod nfo;
mod scraper;

pub const API_BASE_URL: &str = "https://api.themoviedb.org/3";

// 可测试版本的函数，允许注入client和base_url
pub async fn fetch_tv_show_details_with_client(
    client: &Client,
    base_url: &str,
    api_key: &str,
    tv_show_id: u32,
) -> Result<TvShowDetails, reqwest::Error> {
    let url = format!("{}/tv/{}", base_url, tv_show_id);
    client
        .get(&url)
        .query(&[("api_key", api_key)])
        .send()
        .await?
        .json::<TvShowDetails>()
        .await
}

// 可测试版本的函数，允许注入client和base_url
pub async fn search_tv_shows_with_client(
    client: &Client,
    base_url: &str,
    api_key: &str,
    query: &str,
) -> Result<SearchResponse, reqwest::Error> {
    let url = format!("{}/search/tv", base_url);
    client
        .get(&url)
        .query(&[("api_key", api_key), ("query", query)])
        .send()
        .await?
        .json::<SearchResponse>()
        .await
}

pub fn choose_from_results(results: SearchResponse) -> Result<u32, Box<dyn std::error::Error>> {
    use std::io::{self, Write};

    println!("Multiple results found, please choose one:");
    for (i, show) in results.results.iter().enumerate() {
        println!(
            "{}. {} ({})",
            i + 1,
            show.name,
            show.first_air_date.as_deref().unwrap_or("????")
        );
    }

    loop {
        print!("Enter number (1-{}): ", results.results.len());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let choice = input.trim().parse::<usize>();
        if let Ok(choice) = choice {
            if choice > 0 && choice <= results.results.len() {
                return Ok(results.results[choice - 1].id);
            }
        }
        println!("Invalid input. Please try again.");
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

        let _m = mock("GET", format!("/tv/{}", tv_show_id).as_str())
            .match_query(mockito::Matcher::UrlEncoded(
                "api_key".into(),
                api_key.into(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create();

        let client = Client::new();
        let result =
            fetch_tv_show_details_with_client(&client, &mockito::server_url(), api_key, tv_show_id)
                .await;

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
            "results": [
                {
                    "id": 1399,
                    "name": "Game of Thrones",
                    "first_air_date": "2011-04-17",
                    "overview": "The best show ever" // <-- 这行已修改
                }
            ]
        });

        let _m = mock("GET", "/search/tv")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("api_key".into(), api_key.into()),
                mockito::Matcher::UrlEncoded("query".into(), query.into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create()
            .with_body(mock_response.to_string());

        let client = Client::new();
        let result =
            search_tv_shows_with_client(&client, &mockito::server_url(), api_key, query).await;

        assert!(result.is_ok());
        if let Ok(search_response) = result {
            assert_eq!(search_response.results.len(), 1);
            assert_eq!(search_response.results[0].id, 1399);
            assert_eq!(search_response.results[0].name, "Game of Thrones");
            assert_eq!(
                search_response.results[0].overview,
                Some("The best show ever".to_string())
            ); // 增加对 overview 的断言
        }
    }

    #[tokio::test]
    async fn test_fetch_tv_show_details_error() {
        let tv_show_id = 999999; // 不存在的ID
        let api_key = "test_key";

        let _m = mock("GET", format!("/3/tv/{}", tv_show_id).as_str())
            .match_query(mockito::Matcher::UrlEncoded(
                "api_key".into(),
                api_key.into(),
            ))
            .with_status(404)
            .create();

        let client = Client::new();
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
        let url = format!("{}/tv/{}", API_BASE_URL, tv_show_id);
        assert_eq!(url, "https://api.themoviedb.org/3/tv/12345");
    }

    #[tokio::test]
    async fn test_search_tv_shows_url_format() {
        let url = format!("{}/search/tv", API_BASE_URL);
        assert_eq!(url, "https://api.themoviedb.org/3/search/tv");
    }
}
