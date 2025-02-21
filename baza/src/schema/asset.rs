use anyhow::Result;
use serde::{Deserialize, Serialize};

use rs_utils::{get_file_name, get_file_size, get_media_type, Download};

use crate::{
    entities::{BLOBId, Document, DocumentType},
    schema::{Field, FieldType},
    BazaConnection,
};

use super::DataDescription;

pub const ASSET_TYPE: &str = "asset";

pub fn get_asset_definition() -> DataDescription {
    DataDescription {
        document_type: ASSET_TYPE,
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
pub struct AssetData {
    pub filename: String,
    pub media_type: String,
    pub blob: BLOBId,
    pub size: u64,
}

impl AssetData {
    #[must_use]
    pub fn is_image(&self) -> bool {
        self.media_type.starts_with("image/")
    }

    #[must_use]
    pub fn is_audio(&self) -> bool {
        self.media_type.starts_with("audio/")
    }
}

pub type Asset = Document<AssetData>;

pub fn create_asset(
    tx: &mut BazaConnection,
    file_path: &str,
    move_file: bool,
    filename: Option<String>,
) -> Result<Asset> {
    let filename = filename.unwrap_or_else(|| get_file_name(file_path).to_string());

    let media_type = get_media_type(file_path)?;
    let size = get_file_size(file_path)?;

    let blob_id = tx.add_blob(file_path, move_file)?;

    let asset = Document::new_with_data(
        DocumentType::new(ASSET_TYPE),
        AssetData {
            filename,
            media_type,
            size,
            blob: blob_id,
        },
    );

    let mut document = asset.into_document()?;
    tx.stage_document(&mut document, None)?;

    document.convert()
}

pub async fn download_asset(url: &str, tx: &mut BazaConnection) -> Result<Document> {
    let download_result = Download::new(url)?.start().await?;

    let asset = create_asset(
        tx,
        &download_result.file_path,
        true,
        Some(download_result.original_file_name.clone()),
    )?;

    asset.into_document()
}
