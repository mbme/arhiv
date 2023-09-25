use std::fs;

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};

use rs_utils::{file_exists, get_config_home, locate_dominating_file, log};

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    pub arhiv_root: String,

    #[serde(default)]
    pub backup_dir: String,

    #[serde(default = "default_server_port")]
    pub server_port: u16,

    #[serde(default)]
    pub static_peers: Vec<String>,
}

fn default_server_port() -> u16 {
    23421
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
    pub fn must_read() -> (Config, String) {
        Config::read().expect("must be able to read arhiv config")
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
