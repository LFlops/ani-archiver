use crate::tmdb::models::TvShowDetails;

pub fn create_tv_show_nfo(show: &TvShowDetails) -> String {
    let title = &show.name;
    let plot = &show.overview;
    let genres = show
        .genres
        .iter()
        .map(|g| format!("<genre>{}</genre>", g.name))
        .collect::<Vec<_>>()
        .join("\n");
    let year = show
        .first_air_date
        .as_deref()
        .and_then(|d| d.split('-').next())
        .unwrap_or("????");

    format!(
        r#"<?xml version="1.0" encoding="utf-8" standalone="yes"?>
<tvshow>
    <title>{}</title>
    <plot>{}</plot>
    <year>{}</year>
    <premiered>{}</premiered>
    <rating>{}</rating>
    <tmdbid>{}</tmdbid>
    <uniqueid type="tmdb">{}</uniqueid>
    {}
</tvshow>"#,
        title,
        plot,
        year,
        show.first_air_date.as_deref().unwrap_or(""),
        show.vote_average,
        show.id,
        show.id,
        genres
    )
}
