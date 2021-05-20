use std::fs;

use anyhow::*;
use serde::{Deserialize, Serialize};

use rs_utils::{file_exists, get_config_home, locate_dominating_file, log};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    arhiv_root: String,

    #[serde(default)]
    prime_url: String,
}

impl Config {
    pub fn read() -> Result<(Config, String)> {
        let path = find_config_file("arhiv.json")?;
        log::debug!("Found Arhiv config at {}", &path);

        let data = fs::read_to_string(&path)?;

        let config = serde_json::from_str(&data).context("Failed to parse config json")?;

        Ok((config, path))
    }

    pub fn must_read() -> (Config, String) {
        Config::read().expect("must be able to read arhiv config")
    }

    pub fn new(arhiv_root: impl Into<String>, prime_url: impl Into<String>) -> Self {
        Config {
            arhiv_root: arhiv_root.into(),
            prime_url: prime_url.into(),
        }
    }

    pub fn get_prime_url(&self) -> Result<&str> {
        ensure!(!self.prime_url.is_empty(), "prime_url is not set");

        Ok(&self.prime_url)
    }

    pub fn get_root_dir(&self) -> &str {
        &self.arhiv_root
    }
}

// In development, recursively search from current dir upwards for {file_name}
// In production, look up {file_name} in a system config directory
fn find_config_file<S: Into<String>>(file_name: S) -> Result<String> {
    let file_name = file_name.into();

    if cfg!(feature = "production-mode") {
        let config_home = get_config_home().ok_or(anyhow!("Failed to find user config dir"))?;
        let config = format!("{}/{}", config_home, file_name);

        if file_exists(&config).unwrap_or(false) {
            return Ok(config);
        }

        bail!("Can't find Arhiv config at {}", config);
    }

    // in development
    locate_dominating_file(file_name)
}
