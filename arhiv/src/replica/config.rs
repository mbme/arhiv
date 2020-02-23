use anyhow::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaConfig {
    pub arhiv_root: String,
    pub primary_url: String,
}

impl std::str::FromStr for ReplicaConfig {
    type Err = anyhow::Error;

    fn from_str(data: &str) -> Result<ReplicaConfig> {
        serde_json::from_str(data).context("Failed to parse replica config json")
    }
}
