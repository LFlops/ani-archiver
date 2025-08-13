use crate::tmdb::models::{SearchResponse, TvShowDetails};
use reqwest::{Client, Url};
use std::io;
use std::io::{BufRead, Write};

pub mod models;

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

pub async fn check_tmdb_id(
    tmdb_id: &u32,
    client: &Client,
    show_name: &str,
    api_key: &str,
) -> Result<u32, Box<dyn std::error::Error>> {
    if *tmdb_id != 0 {
        return Ok(*tmdb_id);
    }

    // 同时搜索 movie 和 tv
    println!("\nSearching TMDB for '{show_name}'...");

    // 并行搜索电视节目和电影
    let tv_search = search_tv_shows_with_client(client, API_BASE_URL, api_key, show_name);
    let movie_search = search_movies_with_client(client, API_BASE_URL, api_key, show_name);

    let (tv_results, movie_results) = tokio::join!(tv_search, movie_search);

    let mut all_results = Vec::new();

    // 处理电视节目搜索结果
    match tv_results {
        Ok(tv_res) => {
            println!("Found {} TV shows", tv_res.results.len());
            all_results.extend(tv_res.results);
        }
        Err(e) => {
            eprintln!("Error searching for TV shows: {e}");
        }
    }

    // 处理电影搜索结果
    match movie_results {
        Ok(movie_res) => {
            println!("Found {} movies", movie_res.results.len());
            all_results.extend(movie_res.results);
        }
        Err(e) => {
            eprintln!("Error searching for movies: {e}");
        }
    }

    // 如果没有找到任何结果
    if all_results.is_empty() {
        println!("No results found. Skipping.");
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("No TMDB results found for show: {show_name}"),
        )));
    }

    // 创建一个新的SearchResponse包含所有结果
    let search_results = SearchResponse {
        page: 1,
        results: all_results,
        total_pages: 1,
        total_results: 1,
    };

    // todo 通过管道与其他逻辑节藕，避免阻塞整体。

    let mut stdout = io::stdout();
    let stdin = io::stdin();
    let mut buf_reader = io::BufReader::new(stdin);
    choose_from_results(&search_results, &mut buf_reader, &mut stdout)
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

// 可测试版本的函数，允许注入client和base_url
pub async fn search_tv_shows_with_client(
    client: &Client,
    base_url: &str,
    api_key: &str,
    query: &str,
) -> Result<SearchResponse, reqwest::Error> {
    let mut url = Url::parse(base_url).expect("Failed to parse base_url");
    url.path_segments_mut()
        .expect("cannot be base")
        .push("search")
        .push("tv");

    client
        .get(url)
        .query(&[("api_key", api_key), ("query", query)])
        .send()
        .await?
        .json::<SearchResponse>()
        .await
}

// 可测试版本的函数，允许注入client和base_url
pub async fn search_movies_with_client(
    client: &Client,
    base_url: &str,
    api_key: &str,
    query: &str,
) -> Result<SearchResponse, reqwest::Error> {
    let mut url = Url::parse(base_url).expect("Failed to parse base_url");
    url.path_segments_mut()
        .expect("cannot be base")
        .push("search")
        .push("movie");

    client
        .get(url)
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

    #[test]
    fn test_format_search_results() {
        let search_response = get_mock_results();

        let expected_output = "Multiple results found, please choose one:\n1. First Result (2023-01-01)\n2. Second Result (2023-02-02)\n3. Third Result (2023-03-03)\n";
        let actual_output = format_search_results(&search_response);

        assert_eq!(actual_output, expected_output);
    }
    fn get_mock_results() -> SearchResponse {
        SearchResponse {
            page: 1,
            total_pages: 1,
            total_results: 3,
            results: vec![
                TvShowSearchResult {
                    id: 101,
                    name: "First Result".to_string(),
                    first_air_date: Some("2023-01-01".to_string()),
                    overview: Some("Overview 1".to_string()),
                    adult: false,
                    backdrop_path: Some(String::from("...")),
                    genre_ids: vec![1],
                    origin_country: vec![String::from("US")],
                    original_language: String::from("en"),
                    original_name: String::from("First Result"),
                    popularity: 1.0,
                    poster_path: Some(String::from("...")),
                    vote_average: 1.0,
                    vote_count: 1,
                },
                TvShowSearchResult {
                    id: 202,
                    name: "Second Result".to_string(),
                    first_air_date: Some("2023-02-02".to_string()),
                    overview: Some("Overview 2".to_string()),
                    adult: false,
                    backdrop_path: Some(String::from("...")),
                    genre_ids: vec![1],
                    origin_country: vec![String::from("US")],
                    original_language: String::from("en"),
                    original_name: String::from("Second Result"),
                    popularity: 1.0,
                    poster_path: Some(String::from("...")),
                    vote_average: 1.0,
                    vote_count: 1,
                },
                TvShowSearchResult {
                    id: 303,
                    name: "Third Result".to_string(),
                    first_air_date: Some("2023-03-03".to_string()),
                    overview: Some("Overview 3".to_string()),
                    adult: false,
                    backdrop_path: Some(String::from("...")),
                    genre_ids: vec![1],
                    origin_country: vec![String::from("US")],
                    original_language: String::from("en"),
                    original_name: String::from("Third Result"),
                    popularity: 1.0,
                    poster_path: Some(String::from("...")),
                    vote_average: 1.0,
                    vote_count: 1,
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
