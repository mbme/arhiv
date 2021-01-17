use anyhow::*;
use rs_utils::find_config_file;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Config {
    Prime {
        arhiv_id: String,
        arhiv_root: String,

        #[serde(default = "default_server_port")]
        server_port: u16,
    },
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
    pub fn read() -> Result<Config> {
        let path = find_config_file("arhiv.json")?;
        log::debug!("Found Arhiv config at {}", &path);

        let data = fs::read_to_string(&path)?;

        serde_json::from_str(&data).context("Failed to parse config json")
    }

    pub fn must_read() -> Config {
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
