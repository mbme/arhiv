use anyhow::*;
use rs_utils::find_config_file;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub arhiv_root: String,
    pub prime_url: Option<String>,

    #[serde(default = "default_server_port")]
    pub server_port: u16,
}

fn default_server_port() -> u16 {
    8080
}

impl Config {
    pub fn read() -> Result<Config> {
        let path = find_config_file("arhiv.json")?;
        log::debug!("Found Arhiv config at {}", &path);

        let data = fs::read_to_string(&path)?;

        serde_json::from_str(&data).context("Failed to parse config json")
    }

    pub fn must_read() -> Config {
        Config::read().expect("must be able to read arhiv config")
    }
}
