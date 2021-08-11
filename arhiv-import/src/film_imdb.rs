use anyhow::*;
use async_trait::async_trait;
use serde::Deserialize;

use arhiv_core::{entities::Document, Arhiv};
use rs_utils::log;

use crate::utils::{download_file, scrape_and_confirm, Importer};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct IMDBData {
    title: String,
    cover_src: String,
    release_date: String,
    description: String,
    original_language: String,
    countries_of_origin: String,
    creators: String,
    cast: String,
    duration: Option<String>,

    is_series: String,
    number_of_episodes: Option<u64>,
    episode_duration: Option<String>,
}

pub struct IMDBFilmImporter;

#[async_trait]
impl Importer for IMDBFilmImporter {
    fn get_name(&self) -> &str {
        "IMDBFilmImporter"
    }

    fn can_import(&self, url: &str) -> bool {
        url.contains("imdb.com/title/")
    }

    async fn import(&self, url: &str, arhiv: &Arhiv, confirm: bool) -> Result<Document> {
        let data = scrape_and_confirm(url, confirm)?;
        let data: IMDBData = serde_json::from_str(&data).context("failed to parse data")?;

        let cover_file = download_file(&data.cover_src).await?;
        let cover_attachment = arhiv.add_attachment(&cover_file, true)?;
        log::info!("Imported cover {}", &cover_attachment as &Document);

        let mut film = Document::new("film");
        film.data.set("title", data.title);
        film.data.set("cover", cover_attachment.id.to_string());
        film.data.set("release_date", data.release_date);
        film.data.set("description", data.description);
        film.data.set("original_language", data.original_language);
        film.data
            .set("countries_of_origin", data.countries_of_origin);
        film.data.set("creators", data.creators);
        film.data.set("cast", data.cast);
        film.data.set("duration", data.duration);
        film.data.set("is_series", data.is_series == "true");
        film.data.set("number_of_episodes", data.number_of_episodes);
        film.data.set("episode_duration", data.episode_duration);

        arhiv.stage_document(&mut film)?;

        Ok(film)
    }
}
