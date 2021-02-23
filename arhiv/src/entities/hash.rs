use core::fmt;

use anyhow::*;
use rs_utils::get_file_hash_sha256;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct Hash(String);

impl Hash {
    pub fn from_file(file_path: &str) -> Result<Self> {
        let hash = get_file_hash_sha256(&file_path)?;

        Ok(Hash(hash))
    }

    pub fn from_string(hash: String) -> Self {
        Hash(hash)
    }
}

impl From<Hash> for String {
    fn from(value: Hash) -> Self {
        value.0
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sha256:{}", self.0)
    }
}
