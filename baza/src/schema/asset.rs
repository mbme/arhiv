use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use rs_utils::Download;

use crate::{
    baza2::Baza,
    entities::{BLOBId, Document},
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

pub async fn download_asset(url: &str, baza: &mut Baza) -> Result<Asset> {
    let download_result = Download::new(url)?.start().await?;

    let mut asset = baza.create_asset(&download_result.file_path)?;
    asset.data.filename = download_result.original_file_name.clone();

    let document = asset.into_document()?;
    let document = baza
        .stage_document(document, &None)
        .context("Failed to update asset filename")?;

    document.clone().convert()
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
