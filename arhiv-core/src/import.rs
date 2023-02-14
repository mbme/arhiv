use anyhow::{bail, ensure, Context, Result};

use baza::entities::{Document, DocumentClass, DocumentData};
use rs_utils::{ensure_file_exists, remove_file_extension};

use crate::{
    definitions::{ATTACHMENT_TYPE, TRACK_TYPE},
    Arhiv, BazaConnectionExt,
};

impl Arhiv {
    pub fn import_document_from_file(
        &self,
        document_type: &str,
        file_path: &str,
        move_file: bool,
    ) -> Result<Document> {
        ensure_file_exists(file_path)?;

        match document_type {
            TRACK_TYPE => self.import_track(file_path, move_file),
            ATTACHMENT_TYPE => {
                let mut tx = self.baza.get_tx()?;
                let attachment = tx.create_attachment(file_path, move_file)?;
                tx.commit()?;

                attachment.into_document()
            }
            other => bail!("Don't know how to import document of type '{}'", other),
        }
    }

    fn import_track(&self, file_path: &str, move_file: bool) -> Result<Document> {
        let mut tx = self.baza.get_tx()?;

        let attachment = tx.create_attachment(file_path, move_file)?;

        ensure!(
            attachment.data.is_audio(),
            "file type must be audio, got {}",
            attachment.data.media_type
        );

        let file_name = remove_file_extension(&attachment.data.filename)?;

        let mut iter = file_name.split('-');

        let artist = iter
            .next()
            .context("couldn't extract track artist from filename")?;

        let title = iter
            .next()
            .context("couldn't extract track title from filename")?;

        let mut data = DocumentData::new();
        data.set("artist", artist);
        data.set("title", title);
        data.set("track", &attachment.id);

        let class = DocumentClass::new(TRACK_TYPE, "");
        let mut document = Document::new_with_data(class, data);

        tx.stage_document(&mut document)?;

        tx.commit()?;

        Ok(document)
    }
}
