use crate::utils::deserialize_null_string;
use clap::Parser;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

const TV_SHOW_NFO_TEMPLATE: &str = r#"<?xml version="1.0" encoding="utf-8" standalone="yes"?>
<tvshow>
    <title>{}</title>
    <originaltitle>{}</originaltitle>
    <plot>{}</plot>
    <year>{}</year>
    <uniqueid type="tmdb">{}</uniqueid>
</tvshow>"#;
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

impl SearchMultiResult {
    pub fn to_tv_show_nfo(&self) -> Option<String> {
        if self.media_type != MediaType::TV {
            return None;
        }
        let year = self.first_air_date
            .as_deref()
            .and_then(|s| s.split('-').next())
            .unwrap_or("");
        Some(format!(
            TV_SHOW_NFO_TEMPLATE,
            self.name,
            self.original_name,
            self.overview,
            year,
            self.id
        ))
    }
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
