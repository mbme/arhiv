use std::{fs, time::Duration};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};

use rs_utils::{file_exists, get_config_home, locate_dominating_file, log};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    pub arhiv_root: String,

    #[serde(default = "default_server_port")]
    pub server_port: u16,

    #[serde(default = "default_auto_commit")]
    pub auto_commit: bool,

    #[serde(default = "default_auto_commit_delay_in_seconds")]
    pub auto_commit_delay_in_seconds: u64,

    #[serde(default = "default_auto_sync")]
    pub auto_sync: bool,

    #[serde(default = "default_auto_sync_delay_in_seconds")]
    pub auto_sync_delay_in_seconds: u64,
}

fn default_server_port() -> u16 {
    23421
}

fn default_auto_commit() -> bool {
    true
}

fn default_auto_commit_delay_in_seconds() -> u64 {
    600
}

fn default_auto_sync() -> bool {
    true
}

fn default_auto_sync_delay_in_seconds() -> u64 {
    20
}

impl Config {
    pub fn read() -> Result<(Config, String)> {
        let path = find_config_file("arhiv.json")?;
        log::debug!("Found Arhiv config at {}", &path);

        let data = fs::read_to_string(&path)?;

        let config = serde_json::from_str(&data).context("Failed to parse config json")?;

        Ok((config, path))
    }

    #[must_use]
    pub fn get_auto_sync_delay(&self) -> Duration {
        Duration::from_secs(self.auto_sync_delay_in_seconds)
    }

    #[must_use]
    pub fn get_auto_commit_delay(&self) -> Duration {
        Duration::from_secs(self.auto_commit_delay_in_seconds)
    }
}

// In development, recursively search from current dir upwards for {file_name}
// In production, look up {file_name} in a system config directory
fn find_config_file<S: Into<String>>(file_name: S) -> Result<String> {
    let file_name = file_name.into();

    if cfg!(feature = "production-mode") {
        let config_home =
            get_config_home().ok_or_else(|| anyhow!("Failed to find user config dir"))?;
        let config = format!("{config_home}/{file_name}");

        if file_exists(&config).unwrap_or(false) {
            return Ok(config);
        }

        bail!("Can't find Arhiv config at {}", config);
    }

    // in development
    locate_dominating_file(file_name)
}
