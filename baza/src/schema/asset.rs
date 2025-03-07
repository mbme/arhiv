use anyhow::Result;
use serde::{Deserialize, Serialize};

use rs_utils::{get_file_name, get_file_size, get_media_type, Download};

use crate::{
    baza2::Baza,
    entities::{BLOBId, Document, DocumentType},
    schema::{Field, FieldType},
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

pub fn create_asset(baza: &mut Baza, file_path: &str, filename: Option<String>) -> Result<Asset> {
    let filename = filename.unwrap_or_else(|| get_file_name(file_path).to_string());

    let media_type = get_media_type(file_path)?;
    let size = get_file_size(file_path)?;

    let blob_id = baza.add_blob(file_path)?;

    let asset = Document::new_with_data(
        DocumentType::new(ASSET_TYPE),
        AssetData {
            filename,
            media_type,
            size,
            blob: blob_id,
        },
    );

    let document = asset.into_document()?;
    let document = baza.stage_document(document, &None)?.clone();

    document.convert()
}

pub async fn download_asset(url: &str, baza: &mut Baza) -> Result<Asset> {
    let download_result = Download::new(url)?.start().await?;

    let asset = create_asset(
        baza,
        &download_result.file_path,
        Some(download_result.original_file_name.clone()),
    )?;

    Ok(asset)
}

pub fn get_asset_by_blob_id(baza: &Baza, blob_id: &BLOBId) -> Option<Asset> {
    baza.iter_documents()
        .filter_map(|head| {
            let doc = head.get_single_document();

            if doc.document_type.as_ref() == ASSET_TYPE
                && doc.data.get_mandatory_str("blob") == blob_id.as_ref()
            {
                let asset: Asset = doc.clone().convert().expect("must convert to Asset");
                Some(asset)
            } else {
                None
            }
        })
        .next()
}
