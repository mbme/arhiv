use anyhow::*;
use rs_utils::log;
use serde::Deserialize;

use arhiv_core::{entities::Document, Arhiv};

use crate::utils::{confirm_if_needed, scrape};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct YakabooData {
    title: String,
    cover_src: String,
    description: String,
    authors: String,
    language: Option<String>,
    publication_date: String,
    translators: String,
    publisher: String,
    pages: String,

    #[serde(rename = "ISBN")]
    isbn: String,
}

pub async fn import_book_from_yakaboo(url: &str, arhiv: &Arhiv, confirm: bool) -> Result<bool> {
    if !url.contains("www.yakaboo.ua/ua/") {
        return Ok(false);
    }

    let data = scrape(url).context("scrape failed")?;
    let data: YakabooData = serde_json::from_str(&data).context("failed to parse data")?;

    log::info!("Book metadata:\n{:#?}", data);

    if !confirm_if_needed(confirm)? {
        return Ok(true);
    }

    let cover_attachment = arhiv.add_attachment_by_url(&data.cover_src).await?;

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

    // arhiv.stage_document(book)?;

    Ok(true)
}
