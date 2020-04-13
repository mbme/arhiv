use anyhow::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub arhiv_root: String,
    pub primary_url: Option<String>,

    #[serde(default = "default_server_port")]
    pub server_port: u16,
}

fn default_server_port() -> u16 {
    8080
}

impl std::str::FromStr for Config {
    type Err = anyhow::Error;

    fn from_str(data: &str) -> Result<Config> {
        serde_json::from_str(data).context("Failed to parse config json")
    }
}
