use crate::utils::deserialize_null_string;
use clap::Parser;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// A tool to scrape, organize, and create hard links for TV shows.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Source directory to find TV shows.
    #[clap(short, long, value_parser)]
    pub source: PathBuf,

    /// Destination directory to save hard links and metadata.
    #[clap(short, long, value_parser)]
    pub dest: PathBuf,

    /// Force TMDB search even if files are unchanged.
    #[clap(short, long)]
    pub force: bool,
}

#[derive(Debug, Deserialize)]
pub struct Genre {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct TvShowDetails {
    pub id: u32,
    pub name: String,
    #[serde(deserialize_with = "deserialize_null_string")]
    pub overview: String,
    #[serde(default)]
    pub genres: Vec<Genre>,
    pub first_air_date: Option<String>,
    pub vote_average: f64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SearchMultiResult {
    pub id: u32,
    pub title: String,
    pub name: String,
    pub original_title: String,
    pub overview: Option<String>,
    pub media_type: String,
    pub first_air_date: Option<String>,
    pub adult: bool,
    pub backdrop_path: Option<String>,
    pub genre_ids: Vec<u32>,
    pub origin_country: Vec<String>,
    pub original_language: String,
    pub original_name: String,
    pub popularity: f64,
    pub poster_path: Option<String>,
    pub vote_average: f64,
    pub vote_count: u32,
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum MediaType {
    #[serde(rename = "movie")]
    MOVIE,
    #[serde(rename = "tv")]
    TV,
}
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SearchResponse<T> {
    pub page: u32,
    pub results: Vec<T>,
    pub total_pages: u32,
    pub total_results: u32,
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
