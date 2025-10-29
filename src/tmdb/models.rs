use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SearchMultiResult {
    pub id: u32,
    pub overview: String,
    pub backdrop_path: String,
    pub poster_path: String,
    pub media_type: MediaType,
    // tv use this
    pub name: Option<String>,
    pub original_name: Option<String>,
    pub first_air_date: Option<String>,
    // movie use this
    pub title: Option<String>,
    pub original_title: Option<String>,
    pub release_date: Option<String>,
}
#[allow(dead_code)]
impl SearchMultiResult {
    pub fn get_name(&self) -> String {
        match self.media_type {
            MediaType::MOVIE => self.title.clone().unwrap_or_default(),
            MediaType::TV => self.name.clone().unwrap_or_default(),
        }
    }
    pub fn get_release_date(&self) -> String {
        match self.media_type {
            MediaType::MOVIE => self.release_date.clone().unwrap_or_default(),
            MediaType::TV => self.first_air_date.clone().unwrap_or_default(),
        }
    }

    pub fn get_original_name(&self) -> String {
        match self.media_type {
            MediaType::MOVIE => self.original_title.clone().unwrap_or_default(),
            MediaType::TV => self.original_name.clone().unwrap_or_default(),
        }
    }
    pub fn get_poster_url(&self) -> String {
        format!(
            "{}{}",
            // todo need check
            "https://image.tmdb.org/t/p/w500",
            self.poster_path.clone()
        )
    }
    pub fn get_backdrop_url(&self) -> String {
        format!(
            "{}{}",
            // todo need check
            "https://image.tmdb.org/t/p/w500",
            self.poster_path.clone()
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum MediaType {
    #[serde(rename = "movie")]
    MOVIE,
    #[serde(rename = "tv")]
    TV,
}

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("请求错误:{0}")]
    REQWEST(#[from] reqwest::Error),

    #[error("API 返回了不可恢复的错误状态码:{0}")]
    UnrecoverableStatus(StatusCode),

    #[error("重试了{max_retries} 次后仍然失败")]
    MaxRetriesExceeded { max_retries: u32 },
}
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SearchResponse<T> {
    pub page: u32,
    pub results: Vec<T>,
    pub total_pages: u32,
    pub total_results: u32,
}
