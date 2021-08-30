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

use anyhow::*;

use arhiv_core::{entities::Id, Arhiv};
use rs_utils::log;

use crate::{
    attachment::AttachmentImporter, book_yakaboo::YakabooBookImporter, film_imdb::IMDBFilmImporter,
    utils::Importer,
};

mod attachment;
mod book_yakaboo;
mod film_imdb;
mod utils;

pub struct ArhivImport {
    arhiv: Arhiv,
    confirm: bool,
}

impl ArhivImport {
    #[must_use]
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

    pub async fn import(self, url: &str) -> Result<Id> {
        log::info!("Importing url {}", url);

        let importers: Vec<Box<dyn Importer>> = vec![
            Box::new(AttachmentImporter), //
            Box::new(YakabooBookImporter),
            Box::new(IMDBFilmImporter),
        ];

        for importer in importers {
            if !importer.can_import(url) {
                continue;
            }

            log::info!("Found importer {}", importer.get_name());

            let document = importer
                .import(url, &self.arhiv, self.confirm)
                .await
                .context("importer failed")?;

            log::info!("Imported {}", document);

            return Ok(document.id);
        }

        bail!("don't know how to import document from url '{}'", url)
    }
}
