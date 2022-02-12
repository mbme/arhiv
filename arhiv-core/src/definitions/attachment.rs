use std::{
    fs,
    ops::{Deref, DerefMut},
};

use anyhow::{ensure, Context, Error, Result};
use serde_json::json;

use rs_utils::{get_file_name, get_mime_type};

use crate::{
    entities::{BLOBId, Document},
    schema::*,
    Arhiv, ArhivTransaction,
};

pub const ATTACHMENT_TYPE: &str = "attachment";

const FIELD_FILENAME: &str = "filename";
const FIELD_MEDIA_TYPE: &str = "media_type";
const FIELD_BLOB: &str = "blob";
const FIELD_SIZE: &str = "size";

pub fn get_attachment_definitions() -> Vec<DataDescription> {
    vec![DataDescription {
        document_type: ATTACHMENT_TYPE,
        collection_of: Collection::None,
        fields: vec![
            Field {
                name: FIELD_FILENAME,
                field_type: FieldType::String {},
                mandatory: true,
                readonly: true,
            },
            Field {
                name: FIELD_MEDIA_TYPE,
                field_type: FieldType::String {},
                mandatory: true,
                readonly: true,
            },
            Field {
                name: FIELD_BLOB,
                field_type: FieldType::BLOBId,
                mandatory: true,
                readonly: true,
            },
            Field {
                name: FIELD_SIZE, // in bytes
                field_type: FieldType::NaturalNumber {},
                mandatory: true,
                readonly: true,
            },
        ],
    }]
}

#[derive(Debug)]
pub struct Attachment(Document);

impl Attachment {
    #[must_use]
    pub fn is_attachment(document: &Document) -> bool {
        document.document_type == ATTACHMENT_TYPE
    }

    #[must_use]
    pub fn new(filename: &str, media_type: &str, blob_id: &BLOBId, size: u64) -> Self {
        let document = Document::new_with_data(
            ATTACHMENT_TYPE.to_string(),
            json!({
                FIELD_FILENAME: filename,
                FIELD_MEDIA_TYPE: media_type,
                FIELD_BLOB: blob_id,
                FIELD_SIZE: size,
            })
            .try_into()
            .expect("failed to serialize data"),
        );

        Attachment(document)
    }

    pub fn create(file_path: &str, move_file: bool, arhiv: &Arhiv) -> Result<Self> {
        let mut tx = arhiv.get_tx()?;

        let attachment = Attachment::create_tx(file_path, move_file, arhiv, &mut tx)?;

        tx.commit()?;

        Ok(attachment)
    }

    pub fn create_tx(
        file_path: &str,
        move_file: bool,
        arhiv: &Arhiv,
        tx: &mut ArhivTransaction,
    ) -> Result<Self> {
        let filename = get_file_name(file_path).to_string();
        let media_type = get_mime_type(file_path)?;
        let size = fs::metadata(file_path)?.len();

        let blob_id = arhiv.tx_add_blob(file_path, move_file, tx)?;

        let mut attachment = Attachment::new(&filename, &media_type, &blob_id, size);

        arhiv
            .tx_stage_document(&mut attachment, tx)
            .context("failed to create attachment")?;

        Ok(attachment)
    }

    #[must_use]
    pub fn get_blob_id(&self) -> BLOBId {
        let blob_id = self.data.get_mandatory_str(FIELD_BLOB);

        BLOBId::from_string(blob_id)
    }

    #[must_use]
    pub fn get_filename(&self) -> &str {
        self.data.get_mandatory_str(FIELD_FILENAME)
    }

    #[must_use]
    pub fn get_media_type(&self) -> &str {
        self.data.get_mandatory_str(FIELD_MEDIA_TYPE)
    }

    #[must_use]
    pub fn is_image(&self) -> bool {
        self.get_media_type().starts_with("image/")
    }
}

impl Deref for Attachment {
    type Target = Document;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Attachment {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Attachment> for Document {
    fn from(val: Attachment) -> Self {
        val.0
    }
}

impl TryInto<Attachment> for Document {
    type Error = Error;

    fn try_into(self) -> Result<Attachment, Self::Error> {
        ensure!(
            Attachment::is_attachment(&self),
            "document {} must be an attachment",
            &self.id,
        );

        Ok(Attachment(self))
    }
}
