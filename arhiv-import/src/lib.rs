use anyhow::*;

use arhiv_core::Arhiv;
use rs_utils::log;

use crate::{
    book_yakaboo::YakabooBookImporter,
    utils::{ask_confirmation, scrape, Importer},
};

mod book_yakaboo;
mod utils;

pub async fn import_document(url: &str, arhiv: &Arhiv, confirm: bool) -> Result<()> {
    let importers: Vec<Box<dyn Importer>> = vec![
        Box::new(YakabooBookImporter), //
    ];

    for importer in importers {
        if !importer.can_import(url) {
            continue;
        }

        let data = scrape(url).context("scrape failed")?;
        log::info!("Scraped data:\n{}", &data);

        if confirm && !ask_confirmation()? {
            return Ok(());
        }

        let document = importer
            .import(&data, arhiv)
            .await
            .context("importer failed")?;

        log::info!("Imported {}", document);

        return Ok(());
    }

    bail!("don't know how to import document from url '{}'", url)
}
