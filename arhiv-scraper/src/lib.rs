#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::module_inception,
    clippy::wildcard_imports,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::non_ascii_literal
)]

mod scraper;

pub use scraper::Scraper;

#[cfg(test)]
mod tests {
    // disable rule due to false positives in insta::assert_json_snapshot!()
    #![allow(clippy::semicolon_if_nothing_returned)]

    use anyhow::{anyhow, Context, Result};
    use serde_json::Value;

    use arhiv_core::{definitions::get_standard_schema, test_arhiv::TestArhiv};
    use rs_utils::log::setup_error_logger;

    use super::Scraper;

    async fn scrape_and_extract(
        url: &str,
        expected_document_type: &str,
        expected_documents_count: usize,
    ) -> Result<Value> {
        setup_error_logger();

        let arhiv = TestArhiv::new_prime_with_schema(get_standard_schema());

        let scraper = Scraper::new(&arhiv)?;
        let documents = scraper.scrape(url).await.context("scrape failed")?;

        assert_eq!(documents.len(), expected_documents_count);

        let document = documents
            .into_iter()
            .find(|document| document.document_type == expected_document_type)
            .ok_or_else(|| anyhow!("can't find document with type {}", expected_document_type))?;

        Ok(document.data.into())
    }

    #[ignore]
    #[tokio::test]
    async fn test_scrape_yakaboo_book() -> Result<()> {
        let result = scrape_and_extract(
            "https://www.yakaboo.ua/ua/stories-of-your-life-and-others.html",
            "book",
            2,
        )
        .await?;

        insta::assert_json_snapshot!(result, {
            ".cover" => "[cover_attachment_id]",
        });

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_scrape_imdb_film() -> Result<()> {
        let result = scrape_and_extract("https://www.imdb.com/title/tt0133093/", "film", 2).await?;

        insta::assert_json_snapshot!(result, {
            ".cover" => "[cover_attachment_id]",
        });

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_scrape_imdb_series() -> Result<()> {
        let result = scrape_and_extract("https://www.imdb.com/title/tt0098936/", "film", 2).await?;

        insta::assert_json_snapshot!(result, {
            ".cover" => "[cover_attachment_id]",
        });

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_scrape_imdb_mini_series() -> Result<()> {
        let result = scrape_and_extract("https://www.imdb.com/title/tt8134186/", "film", 2).await?;

        insta::assert_json_snapshot!(result, {
            ".cover" => "[cover_attachment_id]",
        });

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_scrape_steam_game() -> Result<()> {
        let result = scrape_and_extract(
            "https://store.steampowered.com/app/814380/Sekiro_Shadows_Die_Twice__GOTY_Edition/",
            "game",
            2,
        )
        .await?;

        insta::assert_json_snapshot!(result, {
            ".cover" => "[cover_attachment_id]",
        });

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_scrape_myanimelist_movie() -> Result<()> {
        let result = scrape_and_extract(
            "https://myanimelist.net/anime/523/Tonari_no_Totoro",
            "film",
            2,
        )
        .await?;

        insta::assert_json_snapshot!(result, {
            ".cover" => "[cover_attachment_id]",
        });

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_scrape_myanimelist_series() -> Result<()> {
        let result = scrape_and_extract(
            "https://myanimelist.net/anime/16498/Shingeki_no_Kyojin",
            "film",
            2,
        )
        .await?;

        insta::assert_json_snapshot!(result, {
            ".cover" => "[cover_attachment_id]",
        });

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_scrape_myanimelist_series_with_prequel() -> Result<()> {
        let result = scrape_and_extract(
            "https://myanimelist.net/anime/40028/Shingeki_no_Kyojin__The_Final_Season",
            "film",
            2,
        )
        .await;

        assert!(result.is_err());

        Ok(())
    }
}
