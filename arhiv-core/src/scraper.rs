use anyhow::{bail, Result};
use serde_json::json;

use rs_utils::{log, Download};
pub use scraper::ScraperOptions;
use scraper::{ScrapedData, Scraper};

use crate::{definitions::Attachment, entities::Document, Arhiv, ArhivConnection};

impl Arhiv {
    pub async fn scrape(
        &self,
        url: impl Into<String>,
        options: ScraperOptions,
    ) -> Result<Vec<Document>> {
        let url = url.into();

        log::info!("Scraping data from '{}'", url);

        let results = tokio::task::spawn_blocking(move || {
            let scraper = Scraper::new_with_options(&options)?;

            scraper.scrape(&url)
        })
        .await??;

        let errors: String = results
            .iter()
            .filter_map(|scrape_result| scrape_result.error.clone())
            .collect::<Vec<_>>()
            .join("\n");

        if !errors.is_empty() {
            bail!("Scraper returned errors: {}", errors);
        }

        let mut tx = self.get_tx()?;
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
                        let cover = create_attachment(cover_url, &mut tx).await?;

                        let mut document = Document::new_with_data(
                            "book",
                            "",
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
                        let cover = create_attachment(cover_url, &mut tx).await?;

                        let mut document = Document::new_with_data(
                            "film",
                            "",
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
                        let cover = create_attachment(cover_url, &mut tx).await?;

                        let mut document = Document::new_with_data(
                            "film",
                            "",
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
                        let cover = create_attachment(cover_url, &mut tx).await?;

                        let mut document = Document::new_with_data(
                            "game",
                            "",
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
                        let image = create_attachment(image_url, &mut tx).await?;
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

async fn create_attachment(url: &str, tx: &mut ArhivConnection) -> Result<Document> {
    let download_result = Download::new(url)?.start().await?;

    let mut attachment = Attachment::create(&download_result.file_path, true, tx)?;
    attachment.data.filename = download_result.original_file_name.clone();

    let mut document = attachment.into_document()?;
    tx.stage_document(&mut document)?;

    Ok(document)
}
