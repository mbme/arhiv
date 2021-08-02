use anyhow::*;
use async_trait::async_trait;

use arhiv_core::{entities::Document, Arhiv};
use rs_utils::log;

use crate::{
    book_yakaboo::YakabooBookImporter,
    utils::{ask_confirmation, scrape},
};

mod book_yakaboo;
mod utils;

#[async_trait]
pub trait Importer {
    fn can_import(&self, url: &str) -> bool;

    async fn import(&self, data: &str, arhiv: &Arhiv) -> Result<Document>;
}

pub async fn run_app(url: &str, arhiv: &Arhiv, confirm: bool) -> Result<()> {
    let importers = vec![Box::new(YakabooBookImporter)];

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
