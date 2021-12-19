use std::{fmt, ops::Deref};

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

use rs_utils::{get_file_hash_blake3, is_valid_base64, to_url_safe_base64};

const BLOB_FILE_PREFIX: &str = "blake3-";
const BLOB_ID_EXPECTED_LENGTH: usize = 44;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct BLOBId(String);

impl fmt::Display for BLOBId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for BLOBId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BLOBId {
    pub fn is_valid_blob_id(id: &str) -> Result<()> {
        ensure!(
            id.len() == BLOB_ID_EXPECTED_LENGTH,
            "BLOB id must be {} chars long",
            BLOB_ID_EXPECTED_LENGTH
        );

        ensure!(is_valid_base64(id), "BLOB id must be a valid base64 string");

        Ok(())
    }

    pub fn from_file(file_path: &str) -> Result<BLOBId> {
        let hash = get_file_hash_blake3(file_path).context("failed to calculate file hash")?;

        let id = to_url_safe_base64(&hash);

        Ok(BLOBId(id))
    }

    #[must_use]
    pub fn get_file_name(&self) -> String {
        format!("{}{}", BLOB_FILE_PREFIX, self.0)
    }

    pub fn from_file_name(file_name: &str) -> Result<BLOBId> {
        ensure!(
            file_name.starts_with(BLOB_FILE_PREFIX),
            "BLOB file name must start with the prefix {}",
            BLOB_FILE_PREFIX
        );
        let blob_id = &file_name[BLOB_FILE_PREFIX.len()..];

        BLOBId::is_valid_blob_id(blob_id)?;

        let blob_id = BLOBId::from_string(blob_id);

        Ok(blob_id)
    }

    #[must_use]
    pub fn from_string(blob_id: impl Into<String>) -> BLOBId {
        BLOBId(blob_id.into())
    }
}
