use anyhow::Result;
use serde::{Deserialize, Serialize, Serializer};

use baza_common::{ExposeSecret, SecretString};

use crate::{
    entities::Document,
    schema::{Field, FieldType},
};

use super::DataDescription;

pub const ASSET_TYPE: &str = "asset";

pub fn get_asset_definition() -> DataDescription {
    DataDescription {
        document_type: ASSET_TYPE,
        title_format: "${filename}",
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
                name: "size", // in bytes
                field_type: FieldType::NaturalNumber {},
                mandatory: true,
                readonly: true,
            },
            Field {
                name: "age_x25519_key",
                field_type: FieldType::String {},
                mandatory: true,
                readonly: true,
            },
        ],
    }
}

fn expose_secret_string<S>(secret: &SecretString, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(secret.expose_secret())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AssetData {
    pub filename: String,
    pub media_type: String,
    pub size: u64,
    #[serde(serialize_with = "expose_secret_string")]
    pub age_x25519_key: SecretString,
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
