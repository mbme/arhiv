use std::{fmt, ops::Deref};

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

use rs_utils::{create_file_reader, get_file_hash_sha256, is_valid_base64, to_url_safe_base64};

const BLOB_PREFIX: &str = "sha256-";
const BLOB_ID_EXPECTED_LENGTH: usize = BLOB_PREFIX.len() + 44;

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
            id.starts_with(BLOB_PREFIX),
            "BLOB id must start with the {BLOB_PREFIX}"
        );

        ensure!(
            id.len() == BLOB_ID_EXPECTED_LENGTH,
            "BLOB id must be {} chars long",
            BLOB_ID_EXPECTED_LENGTH
        );

        let hash = &id[BLOB_PREFIX.len()..];
        ensure!(
            is_valid_base64(hash),
            "BLOB id must be a valid base64 string"
        );

        Ok(())
    }

    pub fn from_file(file_path: &str) -> Result<BLOBId> {
        let reader = create_file_reader(file_path)?;
        let hash = get_file_hash_sha256(reader).context("failed to calculate file hash")?;

        let hash = to_url_safe_base64(&hash);

        let blob_id = BLOBId(format!("{BLOB_PREFIX}{hash}"));

        Ok(blob_id)
    }

    pub fn from_string(value: impl Into<String>) -> Result<BLOBId> {
        let blob_id = value.into();

        BLOBId::is_valid_blob_id(&blob_id)?;

        let blob_id = BLOBId(blob_id);

        Ok(blob_id)
    }
}

#[cfg(test)]
mod tests {
    use rs_utils::workspace_relpath;

    #[test]
    fn test_is_valid_blob_id() {
        use super::BLOBId;

        // Test with a valid BLOB ID
        let src = &workspace_relpath("resources/k2.jpg");
        let id = BLOBId::from_file(src).unwrap();
        assert!(BLOBId::is_valid_blob_id(&id).is_ok());

        // Test with an invalid BLOB ID (wrong prefix)
        let invalid_blob_id_prefix = "sha257-ABCDEFGHIJKLMNOPQRSTUVWXYZabcdef";
        assert!(BLOBId::is_valid_blob_id(invalid_blob_id_prefix).is_err());

        // Test with an invalid BLOB ID (wrong length)
        let invalid_blob_id_length = "sha256-ABCDEFGHIJKLMNOPQRSTUVWXYZabcde";
        assert!(BLOBId::is_valid_blob_id(invalid_blob_id_length).is_err());

        // Test with an invalid BLOB ID (invalid base64)
        let invalid_blob_id_base64 = "sha256-ABCDEFGHIJKLMNOPQRSTUVWXYZabcde!";
        assert!(BLOBId::is_valid_blob_id(invalid_blob_id_base64).is_err());
    }
}
