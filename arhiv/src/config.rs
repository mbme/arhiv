use std::fs;

use anyhow::*;
use serde::{Deserialize, Serialize};

use rs_utils::{file_exists, get_config_home, locate_dominating_file, log::debug};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Config {
    #[serde(rename_all = "camelCase")]
    Prime {
        arhiv_id: String,
        arhiv_root: String,

        #[serde(default = "default_server_port")]
        server_port: u16,
    },

    #[serde(rename_all = "camelCase")]
    Replica {
        arhiv_id: String,
        arhiv_root: String,

        prime_url: String,
    },
}

fn default_server_port() -> u16 {
    8080
}

impl Config {
    pub fn read() -> Result<(Config, String)> {
        let path = find_config_file("arhiv.json")?;
        debug!("Found Arhiv config at {}", &path);

        let data = fs::read_to_string(&path)?;

        let config = serde_json::from_str(&data).context("Failed to parse config json")?;

        Ok((config, path))
    }

    pub fn must_read() -> (Config, String) {
        Config::read().expect("must be able to read arhiv config")
    }

    pub fn is_prime(&self) -> bool {
        match self {
            Config::Prime { .. } => true,
            Config::Replica { .. } => false,
        }
    }

    pub fn get_arhiv_id(&self) -> &str {
        match self {
            Config::Prime { arhiv_id, .. } => arhiv_id,
            Config::Replica { arhiv_id, .. } => arhiv_id,
        }
    }

    pub fn get_root_dir(&self) -> &str {
        match self {
            Config::Prime { arhiv_root, .. } => arhiv_root,
            Config::Replica { arhiv_root, .. } => arhiv_root,
        }
    }

    pub fn get_prime_url(&self) -> Result<&str> {
        match self {
            Config::Prime { .. } => bail!("can't get config.prime_url on prime"),
            Config::Replica { prime_url, .. } => Ok(prime_url),
        }
    }

    pub fn get_server_port(&self) -> Result<u16> {
        match self {
            Config::Prime { server_port, .. } => Ok(*server_port),
            Config::Replica { .. } => bail!("can't get config.server_port on replica"),
        }
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
