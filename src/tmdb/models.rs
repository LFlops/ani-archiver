use crate::utils::deserialize_null_string;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    // #[serde(deserialize_with = "deserialize_null_string")]
    pub overview: String,
    #[serde(default)]
    pub genres: Vec<Genre>,
    pub first_air_date: Option<String>,
    pub vote_average: f64,
}

#[derive(Debug, Deserialize)]
pub struct TvShowSearchResult {
    pub id: u32,
    pub name: String,
    pub first_air_date: Option<String>,
    #[allow(dead_code)]
    pub overview: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<TvShowSearchResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessedMarker {
    pub tmdb_id: u32,
    pub file_hashes: Vec<String>,
}