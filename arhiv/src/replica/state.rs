use crate::entities::*;
use crate::utils::ensure_exists;
use anyhow::*;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StateDTO {
    pub rev: Revision,
}

pub struct StorageState {
    file_path: String,
}

impl StorageState {
    pub fn new(root_path: &str) -> StorageState {
        StorageState {
            file_path: format!("{}/arhiv-state.json", root_path),
        }
    }

    pub fn read(&self) -> Result<StateDTO> {
        let state_str = fs::read_to_string(&self.file_path)?;

        serde_json::from_str(&state_str).context("Failed to parse storage state json")
    }

    pub fn write(&self, state: StateDTO) -> Result<()> {
        let state_str =
            serde_json::to_string(&state).context("Failed to serialize storage state to json")?;

        fs::write(&self.file_path, state_str).context("failed to save storage state")
    }

    pub fn assert_exists(&self) -> Result<()> {
        ensure_exists(&self.file_path, false)
    }
}
