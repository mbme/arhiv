use crate::utils::file_exists;
use anyhow::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub is_prime: bool,
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

fn find_config_file() -> Option<String> {
    // FIXME read global config in prod mode

    let mut dir = env::current_dir().expect("must be able to get current dir");

    loop {
        let config = format!(
            "{}/arhiv.json",
            &dir.to_str().expect("must be able to serialize path")
        );

        if file_exists(&config).unwrap_or(false) {
            return Some(config);
        }

        if let Some(parent) = dir.parent() {
            dir = parent.to_path_buf();
        } else {
            return None;
        }
    }
}

impl Config {
    pub fn read() -> Result<Config> {
        let path = find_config_file().expect("must be able to find arhiv config");

        fs::read_to_string(&path)
            .with_context(|| format!("must be able to read arhiv config at {}", &path))?
            .parse()
            .with_context(|| format!("must be able to parse arhiv config at {}", &path))
    }

    pub fn must_read() -> Config {
        Config::read().expect("must be able to read arhiv config")
    }

    pub fn get_attachment_data_url(&self, id: &str) -> Result<String> {
        let primary_url = self
            .primary_url
            .as_ref()
            .ok_or(anyhow!("config.primary_url is missing"))?;

        Ok(format!("{}/attachment-data/{}", primary_url, id))
    }

    pub fn get_changeset_url(&self) -> Result<String> {
        let primary_url = self
            .primary_url
            .as_ref()
            .ok_or(anyhow!("config.primary_url is missing"))?;

        Ok(format!("{}/changeset", primary_url))
    }
}
