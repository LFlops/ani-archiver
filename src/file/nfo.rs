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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmdb::models::Genre;

    #[test]
    fn test_create_tv_show_nfo_with_complete_data() {
        let show = TvShowDetails {
            id: 1399,
            name: "Game of Thrones".to_string(),
            overview: "Seven noble families fight for control of the mythical land of Westeros."
                .to_string(),
            genres: vec![
                Genre {
                    name: "Sci-Fi".to_string(),
                },
                Genre {
                    name: "Fantasy".to_string(),
                },
            ],
            first_air_date: Some("2011-04-17".to_string()),
            vote_average: 8.4,
        };

        let nfo_content = create_tv_show_nfo(&show);

        assert!(nfo_content.contains("<title>Game of Thrones</title>"));
        assert!(nfo_content.contains(
            "<plot>Seven noble families fight for control of the mythical land of Westeros.</plot>"
        ));
        assert!(nfo_content.contains("<year>2011</year>"));
        assert!(nfo_content.contains("<premiered>2011-04-17</premiered>"));
        assert!(nfo_content.contains("<rating>8.4</rating>"));
        assert!(nfo_content.contains("<tmdbid>1399</tmdbid>"));
        assert!(nfo_content.contains("<uniqueid type=\"tmdb\">1399</uniqueid>"));
        assert!(nfo_content.contains("<genre>Sci-Fi</genre>"));
        assert!(nfo_content.contains("<genre>Fantasy</genre>"));
    }

    #[test]
    fn test_create_tv_show_nfo_with_missing_air_date() {
        let show = TvShowDetails {
            id: 1234,
            name: "Unknown Show".to_string(),
            overview: "A show with no air date.".to_string(),
            genres: vec![],
            first_air_date: None,
            vote_average: 7.5,
        };

        let nfo_content = create_tv_show_nfo(&show);

        assert!(nfo_content.contains("<year>????</year>"));
        assert!(nfo_content.contains("<premiered></premiered>"));
    }

    #[test]
    fn test_create_tv_show_nfo_with_empty_air_date() {
        let show = TvShowDetails {
            id: 5678,
            name: "Incomplete Date Show".to_string(),
            overview: "A show with empty air date.".to_string(),
            genres: vec![Genre {
                name: "Drama".to_string(),
            }],
            first_air_date: None,
            vote_average: 6.2,
        };

        let nfo_content = create_tv_show_nfo(&show);

        assert!(nfo_content.contains("<year>????</year>"));
        assert!(nfo_content.contains("<premiered></premiered>"));
        assert!(nfo_content.contains("<genre>Drama</genre>"));
    }
}
