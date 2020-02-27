use anyhow::*;
use config::PrimeConfig;
use storage::Storage;

mod config;
mod state;
mod storage;

pub struct Prime {
    storage: Storage,
    config: PrimeConfig,
}

impl Prime {
    pub fn open(config: PrimeConfig) -> Prime {
        let root_dir = &config.arhiv_root.clone();
        Prime {
            config,
            storage: Storage::open(root_dir).expect("storage must exist"),
        }
    }

    pub fn create(config: PrimeConfig) -> Result<Prime> {
        let root_dir = &config.arhiv_root.clone();
        Ok(Prime {
            config,
            storage: Storage::create(root_dir)?,
        })
    }
}
