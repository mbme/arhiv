use std::{fs, time::Duration};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    #[serde(default = "default_server_port")]
    pub server_port: u16,

    #[serde(default = "default_auto_commit_delay_in_seconds")]
    pub auto_commit_delay_in_seconds: u64,

    #[serde(default = "default_auto_sync_delay_in_seconds")]
    pub auto_sync_delay_in_seconds: u64,
}

fn default_server_port() -> u16 {
    23421
}

fn default_auto_commit_delay_in_seconds() -> u64 {
    600
}

fn default_auto_sync_delay_in_seconds() -> u64 {
    20
}

impl Config {
    pub fn read(config_path: &str) -> Result<Config> {
        let data = fs::read_to_string(config_path).context("Failed to read config file")?;

        let config = serde_json::from_str(&data).context("Failed to parse config json")?;

        Ok(config)
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
