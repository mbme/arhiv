use anyhow::Result;
use serde::{Deserialize, Serialize};

use rs_utils::{get_file_name, get_file_size, get_media_type, Download};

use crate::{
    entities::{BLOBId, Document, DocumentType},
    schema::{Field, FieldType},
    BazaConnection,
};

use super::DataDescription;

pub const ATTACHMENT_TYPE: &str = "attachment";

pub fn get_attachment_definition() -> DataDescription {
    DataDescription {
        document_type: ATTACHMENT_TYPE,
        title_format: "{filename}",
        fields: vec![
            Field {
                name: "filename",
                field_type: FieldType::String {},
                mandatory: true,
                readonly: false,
            },
            Field {
                name: "media_type",
                field_type: FieldType::String {},
                mandatory: true,
                readonly: false,
            },
            Field {
                name: "blob",
                field_type: FieldType::BLOBId {},
                mandatory: true,
                readonly: true,
            },
            Field {
                name: "size", // in bytes
                field_type: FieldType::NaturalNumber {},
                mandatory: true,
                readonly: true,
            },
        ],
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AttachmentData {
    pub filename: String,
    pub media_type: String,
    pub blob: BLOBId,
    pub size: u64,
}

impl AttachmentData {
    #[must_use]
    pub fn is_image(&self) -> bool {
        self.media_type.starts_with("image/")
    }

    #[must_use]
    pub fn is_audio(&self) -> bool {
        self.media_type.starts_with("audio/")
    }
}

pub type Attachment = Document<AttachmentData>;

pub fn create_attachment(
    tx: &mut BazaConnection,
    file_path: &str,
    move_file: bool,
    filename: Option<String>,
) -> Result<Attachment> {
    let filename = filename.unwrap_or_else(|| get_file_name(file_path).to_string());

    let media_type = get_media_type(file_path)?;
    let size = get_file_size(file_path)?;

    let blob_id = tx.add_blob(file_path, move_file)?;

    let attachment = Document::new_with_data(
        DocumentType::new(ATTACHMENT_TYPE),
        AttachmentData {
            filename,
            media_type,
            size,
            blob: blob_id,
        },
    );

    let mut document = attachment.into_document()?;
    tx.stage_document(&mut document, None)?;

    document.convert()
}

pub async fn download_attachment(url: &str, tx: &mut BazaConnection) -> Result<Document> {
    let download_result = Download::new(url)?.start().await?;

    let attachment = create_attachment(
        tx,
        &download_result.file_path,
        true,
        Some(download_result.original_file_name.clone()),
    )?;

    attachment.into_document()
}
