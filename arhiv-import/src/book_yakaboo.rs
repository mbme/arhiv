use anyhow::*;
use async_trait::async_trait;
use serde::Deserialize;

use arhiv_core::{entities::Document, Arhiv};
use rs_utils::log;

use crate::{
    utils::{download_file, scrape_and_confirm},
    Importer,
};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct YakabooData {
    title: String,
    cover_src: String,
    description: String,
    authors: String,
    language: Option<String>,
    publication_date: String,
    translators: Option<String>,
    publisher: String,
    pages: u64,

    #[serde(rename = "ISBN")]
    isbn: String,
}

pub struct YakabooBookImporter;

#[async_trait]
impl Importer for YakabooBookImporter {
    fn get_name(&self) -> &str {
        "YakabooBookImporter"
    }

    fn can_import(&self, url: &str) -> bool {
        url.contains("www.yakaboo.ua/ua/")
    }

    async fn import(&self, url: &str, arhiv: &Arhiv, confirm: bool) -> Result<Document> {
        let data = scrape_and_confirm(url, confirm)?;
        let data: YakabooData = serde_json::from_str(&data).context("failed to parse data")?;

        let mut tx = arhiv.get_tx()?;

        let cover_file = download_file(&data.cover_src).await?;
        let cover_attachment = arhiv.tx_add_attachment(&cover_file, true, &mut tx)?;
        log::info!("Imported cover {}", &cover_attachment as &Document);

        let mut book = Document::new("book");
        book.data.set("title", data.title);
        book.data.set("cover", cover_attachment.id.to_string());
        book.data.set("description", data.description);
        book.data.set("authors", data.authors);
        book.data.set("language", data.language);
        book.data.set("publication_date", data.publication_date);
        book.data.set("translators", data.translators);
        book.data.set("publisher", data.publisher);
        book.data.set("pages", data.pages);
        book.data.set("ISBN", data.isbn);

        arhiv.tx_stage_document(&mut book, &mut tx)?;

        tx.commit()?;

        Ok(book)
    }
}
