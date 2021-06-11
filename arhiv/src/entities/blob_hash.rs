use core::fmt;

use anyhow::*;
use rs_utils::get_file_hash_sha256;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct BLOBHash(String);

impl BLOBHash {
    pub fn from_file(file_path: &str) -> Result<Self> {
        let hash = get_file_hash_sha256(&file_path)?;

        Ok(BLOBHash(hash))
    }

    pub fn from_string(hash: impl Into<String>) -> Self {
        BLOBHash(hash.into())
    }
}

impl fmt::Display for BLOBHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for BLOBHash {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
