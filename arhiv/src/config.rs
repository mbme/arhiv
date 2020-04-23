use anyhow::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub prime: bool,
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

impl Config {
    pub fn read() -> Result<Config> {
        let path = &format!(
            "{}/arhiv.json",
            env::var("CARGO_MANIFEST_DIR").context("env var CARGO_MANIFEST_DIR must be set")?
        );

        fs::read_to_string(path)
            .with_context(|| format!("must be able to read arhiv config at {}", path))?
            .parse()
            .with_context(|| format!("must be able to parse arhiv config at {}", path))
    }
}
