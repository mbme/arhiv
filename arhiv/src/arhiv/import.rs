use anyhow::{bail, ensure, Context, Result};

use baza::{
    entities::{Document, DocumentData, DocumentType},
    schema::ASSET_TYPE,
};
use rs_utils::{ensure_file_exists, remove_file_extension, remove_file_if_exists};

use crate::{definitions::TRACK_TYPE, Arhiv};

impl Arhiv {
    pub fn import_document_from_file(
        &self,
        document_type: &str,
        file_path: &str,
        remove_original: bool,
    ) -> Result<Document> {
        ensure_file_exists(file_path)?;

        match document_type {
            TRACK_TYPE => self.import_track(file_path, remove_original),
            ASSET_TYPE => {
                let mut baza = self.baza.open_mut()?;
                let asset = baza.create_asset(file_path)?;
                baza.save_changes()?;

                if remove_original {
                    remove_file_if_exists(file_path)?;
                }

                asset.into_document()
            }
            other => bail!("Don't know how to import document of type '{}'", other),
        }
    }

    fn import_track(&self, file_path: &str, remove_original: bool) -> Result<Document> {
        let mut baza = self.baza.open_mut()?;

        let asset = baza.create_asset(file_path)?;

        ensure!(
            asset.data.is_audio(),
            "file type must be audio, got {}",
            asset.data.media_type
        );

        let file_name = remove_file_extension(&asset.data.filename)?;

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
        data.set("track", &asset.id);

        let document = Document::new_with_data(DocumentType::new(TRACK_TYPE), data);

        let document = baza.stage_document(document, &None)?.clone();

        baza.save_changes()?;

        if remove_original {
            remove_file_if_exists(file_path)?;
        }

        Ok(document)
    }
}
