use crate::tmdb::models::{SearchResponse, TvShowDetails};
use reqwest::Client;
use std::io::{BufRead, Write};

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

pub fn format_search_results(results: &SearchResponse) -> String {
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
pub fn parse_choice(input: &str, results: &SearchResponse) -> Result<u32, &'static str> {
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
    results: &SearchResponse,
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
        reader.read_line(&mut input)?; // Read from the provided reader.

        // Use our pure logic function to process the input.
        match parse_choice(&input, results) {
            Ok(id) => return Ok(id), // Valid choice, exit the loop and return.
            Err(msg) => writeln!(writer, "{}", msg)?, // Invalid choice, print error and loop again.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmdb::models::TvShowSearchResult;
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
            "results": [
                {
                    "id": 1399,
                    "name": "Game of Thrones",
                    "first_air_date": "2011-04-17",
                    "overview": "The best show ever"
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
            .create();

        let client = Client::builder()
            .no_proxy()
            .build()
            .expect("Client build failed");
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
            );
        }
    }

    #[tokio::test]
    async fn test_fetch_tv_show_details_error() {
        let tv_show_id = 999999; // 不存在的ID
        let api_key = "test_key";

        let _m = mock("GET", format!("/tv/{}", tv_show_id).as_str())
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
        let url = format!("{}/tv/{}", API_BASE_URL, tv_show_id);
        assert_eq!(url, "https://api.themoviedb.org/3/tv/12345");
    }

    #[tokio::test]
    async fn test_search_tv_shows_url_format() {
        let url = format!("{}/search/tv", API_BASE_URL);
        assert_eq!(url, "https://api.themoviedb.org/3/search/tv");
    }

    #[test]
    fn test_format_search_results() {
        let search_response = SearchResponse {
            results: vec![
                models::TvShowSearchResult {
                    id: 1,
                    name: "Show 1".to_string(),
                    first_air_date: Some("2022-01-01".to_string()),
                    overview: Some("Overview 1".to_string()),
                },
                models::TvShowSearchResult {
                    id: 2,
                    name: "Show 2".to_string(),
                    first_air_date: None,
                    overview: Some("Overview 2".to_string()),
                },
            ],
        };

        let expected_output = "Multiple results found, please choose one:\n1. Show 1 (2022-01-01)\n2. Show 2 (????)\n";
        let actual_output = format_search_results(&search_response);

        assert_eq!(actual_output, expected_output);
    }
    fn get_mock_results() -> SearchResponse {
        SearchResponse {
            results: vec![
                TvShowSearchResult {
                    id: 101,
                    name: "First Result".to_string(),
                    first_air_date: Some("2023-01-01".to_string()),
                    overview: Some("Overview 1".to_string()),
                },
                TvShowSearchResult {
                    id: 202,
                    name: "Second Result".to_string(),
                    first_air_date: Some("2023-02-02".to_string()),
                    overview: Some("Overview 2".to_string()),
                },
                TvShowSearchResult {
                    id: 303,
                    name: "Third Result".to_string(),
                    first_air_date: Some("2023-03-03".to_string()),
                    overview: Some("Overview 3".to_string()),
                },
            ],
        }
    }
    #[test]
    fn test_parse_choice_valid() {
        let results = get_mock_results();
        // A valid choice "2" should return the ID of the second item.
        assert_eq!(parse_choice("2\n", &results), Ok(202));
    }

    #[test]
    fn test_parse_choice_with_whitespace() {
        let results = get_mock_results();
        // Input with extra whitespace should be trimmed and parsed correctly.
        assert_eq!(parse_choice("  1  ", &results), Ok(101));
    }

    #[test]
    fn test_parse_choice_out_of_bounds() {
        let results = get_mock_results();
        // "4" is a number but is not a valid choice.
        assert_eq!(parse_choice("4", &results), Err("Choice is out of range."));
        // "0" is also out of bounds.
        assert_eq!(parse_choice("0", &results), Err("Choice is out of range."));
    }

    #[test]
    fn test_parse_choice_invalid_input() {
        let results = get_mock_results();
        // Non-numeric input should result in an error.
        assert_eq!(
            parse_choice("abc", &results),
            Err("Invalid input. Please enter a number.")
        );
    }

    // --- Tests for the I/O function: `choose_from_results` ---

    #[test]
    fn test_choose_from_results_happy_path() {
        let results = get_mock_results();
        // Simulate user input "2\n". The `b` prefix creates a byte slice.
        let mut mock_reader = std::io::Cursor::new(b"2\n");
        // Capture output in a vector of bytes.
        let mut mock_writer = Vec::new();

        // Run the function with our mock I/O objects.
        let result_id = choose_from_results(&results, &mut mock_reader, &mut mock_writer).unwrap();

        // Check if the correct ID was returned.
        assert_eq!(result_id, 202);

        // Check if the output written to the console is correct.
        let output = String::from_utf8(mock_writer).unwrap();
        assert!(output.contains("Multiple results found, please choose one:"));
        assert!(output.contains("2. Second Result"));
        assert!(output.contains("Enter number (1-3):"));
    }

    #[test]
    fn test_choose_from_results_invalid_then_valid_input() {
        let results = get_mock_results();
        // Simulate a user first typing "bad", then typing a valid choice "3".
        let mut mock_reader = std::io::Cursor::new(b"bad\n3\n");
        let mut mock_writer = Vec::new();

        let result_id = choose_from_results(&results, &mut mock_reader, &mut mock_writer).unwrap();

        // The final result should be the valid choice.
        assert_eq!(result_id, 303);

        // Check the output to ensure the error message was displayed before success.
        let output = String::from_utf8(mock_writer).unwrap();
        assert!(output.contains("Invalid input. Please enter a number.")); // Error message
        assert!(output.contains("Enter number (1-3):")); // Prompt appears twice
    }
}
