use anyhow::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrimeConfig {
    pub arhiv_root: String,
    pub port: u8,
}

impl std::str::FromStr for PrimeConfig {
    type Err = anyhow::Error;

    fn from_str(data: &str) -> Result<PrimeConfig> {
        serde_json::from_str(data).context("Failed to parse prime config json")
    }
}
