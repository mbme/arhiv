use anyhow::{bail, Context, Result};
use serde_json::json;

use baza::{
    entities::{Document, DocumentClass},
    BazaConnection,
};
use rs_utils::{log, Download};
pub use scraper::{ScrapedData, ScraperOptions};

use crate::{
    definitions::{BOOK_TYPE, FILM_TYPE, GAME_TYPE},
    Arhiv, BazaConnectionExt,
};

impl Arhiv {
    pub async fn scrape(
        &self,
        url: impl Into<String>,
        scraper_options: ScraperOptions,
    ) -> Result<Vec<Document>> {
        let url = url.into();

        log::info!("Scraping data from '{}'", url);

        let results = scraper_options
            .scrape(&url)
            .await
            .context("failed to scrape url")?;

        let errors: String = results
            .iter()
            .filter_map(|scrape_result| scrape_result.error.clone())
            .collect::<Vec<_>>()
            .join("\n");

        if !errors.is_empty() {
            bail!("Scraper returned errors: {}", errors);
        }

        let mut tx = self.baza.get_tx()?;
        let mut documents = Vec::new();

        for scrape_result in results {
            if let Some(ref data) = scrape_result.data {
                match data {
                    ScrapedData::YakabooBook {
                        cover_url,
                        title,
                        authors,
                        publication_date,
                        description,
                        translators,
                        publisher,
                        pages,
                        language,
                    } => {
                        let cover = download_attachment(cover_url, &mut tx).await?;

                        let mut document = Document::new_with_data(
                            DocumentClass::new(BOOK_TYPE, ""),
                            json!({
                                "cover": &cover.id,
                                "title": title,
                                "description": description,
                                "authors": authors,
                                "language": language,
                                "publication_date": publication_date,
                                "translators": translators,
                                "publisher": publisher,
                                "pages": pages,
                            })
                            .try_into()?,
                        );

                        tx.stage_document(&mut document)?;

                        documents.push(cover);
                        documents.push(document);
                    }
                    ScrapedData::IMDBFilm {
                        title,
                        cover_url,
                        release_date,
                        original_language,
                        countries_of_origin,
                        creators,
                        cast,
                        seasons,
                        episodes,
                        duration,
                        description,
                    } => {
                        let cover = download_attachment(cover_url, &mut tx).await?;

                        let mut document = Document::new_with_data(
                            DocumentClass::new(FILM_TYPE, ""),
                            json!({
                                "cover": &cover.id,
                                "title": title,
                                "release_date": release_date,
                                "original_language": original_language,
                                "countries_of_origin": countries_of_origin,
                                "description": description,
                                "duration": duration,
                                "seasons": seasons,
                                "episodes": episodes,
                                "creators": creators,
                                "cast": cast,
                            })
                            .try_into()?,
                        );

                        tx.stage_document(&mut document)?;

                        documents.push(cover);
                        documents.push(document);
                    }
                    ScrapedData::MyAnimeListAnime {
                        title,
                        cover_url,
                        release_date,
                        creators,
                        duration,
                        description,
                    } => {
                        let cover = download_attachment(cover_url, &mut tx).await?;

                        let mut document = Document::new_with_data(
                            DocumentClass::new(FILM_TYPE, ""),
                            json!({
                                "cover": &cover.id,
                                "title": title,
                                "release_date": release_date,
                                "original_language": "Japanese",
                                "countries_of_origin": "Japan",
                                "creators": creators,
                                "duration": duration,
                                "description": description,
                            })
                            .try_into()?,
                        );

                        tx.stage_document(&mut document)?;

                        documents.push(cover);
                        documents.push(document);
                    }
                    ScrapedData::SteamGame {
                        cover_url,
                        name,
                        release_date,
                        developers,
                        description,
                    } => {
                        let cover = download_attachment(cover_url, &mut tx).await?;

                        let mut document = Document::new_with_data(
                            DocumentClass::new(GAME_TYPE, ""),
                            json!({
                                "cover": &cover.id,
                                "name": name,
                                "release_date": release_date,
                                "developers": developers,
                                "description": description,
                            })
                            .try_into()?,
                        );

                        tx.stage_document(&mut document)?;

                        documents.push(cover);
                        documents.push(document);
                    }
                    ScrapedData::Image { image_url } => {
                        let image = download_attachment(image_url, &mut tx).await?;
                        documents.push(image);
                    }
                    _ => {
                        log::warn!(
                            "Got unsupported scraper result, ignoring: {:?}",
                            scrape_result
                        );
                    }
                }
            }
        }

        tx.commit()?;

        Ok(documents)
    }
}

async fn download_attachment(url: &str, tx: &mut BazaConnection) -> Result<Document> {
    let download_result = Download::new(url)?.start().await?;

    let mut attachment = tx.create_attachment(&download_result.file_path, true)?;
    attachment.data.filename = download_result.original_file_name.clone();

    let mut document = attachment.into_document()?;
    tx.stage_document(&mut document)?;

    Ok(document)
}
