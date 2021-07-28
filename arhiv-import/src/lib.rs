use anyhow::*;

use arhiv_core::Arhiv;

use crate::book_yakaboo::import_book_from_yakaboo;

mod book_yakaboo;
mod utils;

type Importer = fn(&str, &Arhiv, bool) -> Result<bool>;

pub fn run_app(url: &str, arhiv: &Arhiv, confirm: bool) -> Result<()> {
    let importers: Vec<Importer> = vec![import_book_from_yakaboo];

    for importer in importers {
        let imported = importer(url, arhiv, confirm).context("importer failed")?;

        if imported {
            return Ok(());
        }
    }

    bail!("don't know how to import document from url '{}'", url)
}
