use crate::entities::*;
use anyhow::*;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StateDTO {
    pub replica_rev: Revision,
}

impl StateDTO {
    fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize replica storage state to json")
    }
}

impl std::str::FromStr for StateDTO {
    type Err = anyhow::Error;

    fn from_str(data: &str) -> Result<StateDTO> {
        serde_json::from_str(data).context("Failed to parse replica storage state json")
    }
}

pub struct StorageState {
    file_path: String,
}

impl StorageState {
    pub fn new(file_path: String) -> StorageState {
        StorageState { file_path }
    }

    pub fn read(&self) -> Result<StateDTO> {
        fs::read_to_string(&self.file_path)?.parse()
    }

    pub fn write(&self, state: StateDTO) -> Result<()> {
        fs::write(&self.file_path, state.serialize()).context("failed to save storage state")
    }
}
