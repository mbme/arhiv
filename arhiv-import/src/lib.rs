use anyhow::*;
use dialoguer::{theme::ColorfulTheme, Confirm};

use arhiv_core::{entities::Id, Arhiv};
use rs_utils::log;

use crate::{
    book_yakaboo::YakabooBookImporter,
    utils::{scrape, Importer},
};

mod book_yakaboo;
mod utils;

pub struct ArhivImport {
    arhiv: Arhiv,
    confirm: bool,
}

impl ArhivImport {
    pub fn new(arhiv: Arhiv) -> Self {
        ArhivImport {
            arhiv,
            confirm: true,
        }
    }

    pub fn confirm(&mut self, confirm: bool) -> &mut Self {
        self.confirm = confirm;

        self
    }

    fn import_confirmed(&self) -> Result<bool> {
        if !self.confirm {
            return Ok(true);
        }

        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you really want to continue?")
            .default(true)
            .interact()
            .context("failed to ask confirmation")
    }

    pub async fn import(self, url: &str) -> Result<Option<Id>> {
        log::info!("Importing url {}", url);

        let importers: Vec<Box<dyn Importer>> = vec![
            Box::new(YakabooBookImporter), //
        ];

        for importer in importers {
            if !importer.can_import(url) {
                continue;
            }

            log::info!("Found importer {}", importer.get_name());

            log::info!("Scraping data ...");
            let data = scrape(url).context("scrape failed")?;
            log::info!("Scraped data:\n{}", &data);

            if !self.import_confirmed()? {
                return Ok(None);
            }

            let document = importer
                .import(&data, &self.arhiv)
                .await
                .context("importer failed")?;

            log::info!("Imported {}", document);

            return Ok(Some(document.id));
        }

        bail!("don't know how to import document from url '{}'", url)
    }
}
