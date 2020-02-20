use crate::entities::*;
use anyhow::*;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StorageState {
    pub replica_rev: Revision,
}

impl StorageState {
    fn parse(src: &str) -> Result<StorageState> {
        serde_json::from_str(src).context("Failed to parse storage state json")
    }

    fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize document to json")
    }
}

impl std::str::FromStr for StorageState {
    type Err = anyhow::Error;

    fn from_str(data: &str) -> Result<StorageState> {
        StorageState::parse(data)
    }
}

impl StorageState {
    pub fn new() -> StorageState {
        StorageState { replica_rev: 0 }
    }

    pub fn ensure_exists(path: &str) -> Result<()> {
        match fs::metadata(path) {
            Ok(metadata) if !metadata.is_file() => {
                return Err(anyhow!("path isn't a file: {}", path));
            }

            Ok(_) => Ok(()),

            Err(_) => Err(anyhow!("path doesn't exist {}", path)),
        }
    }

    pub fn read(path: &str) -> Result<StorageState> {
        fs::read_to_string(path)?.parse()
    }

    pub fn write(&self, path: &str) -> Result<()> {
        fs::write(path, self.serialize()).context("failed to save storage state")
    }
}
