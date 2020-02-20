use anyhow::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ArhivConfig {
    pub arhiv_root: String,
    pub primary_url: String,
}

impl std::str::FromStr for ArhivConfig {
    type Err = anyhow::Error;

    fn from_str(data: &str) -> Result<ArhivConfig> {
        serde_json::from_str(data).context("Failed to parse arhiv config json")
    }
}
