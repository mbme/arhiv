use crate::utils::find_config_file;
use anyhow::*;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize)]
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

    pub fn get_attachment_data_url(&self, id: &str) -> Result<String> {
        let prime_url = self
            .prime_url
            .as_ref()
            .ok_or(anyhow!("config.prime_url is missing"))?;

        Ok(format!("{}/attachment-data/{}", prime_url, id))
    }

    pub fn get_changeset_url(&self) -> Result<String> {
        let prime_url = self
            .prime_url
            .as_ref()
            .ok_or(anyhow!("config.prime_url is missing"))?;

        Ok(format!("{}/changeset", prime_url))
    }
}
