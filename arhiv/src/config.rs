use std::time::Duration;

use anyhow::Result;

use baza::{BazaConnection, KvsConstKey};

use crate::Arhiv;

const DEFAULT_SERVER_PORT: u16 = 23421;
const DEFAULT_AUTO_COMMIT_DELAY_IN_SECONDS: u64 = 600;
const DEFAULT_AUTO_SYNC_DELAY_IN_SECONDS: u64 = 20;

const CONFIG_NAMESPACE: &str = "arhiv-config";

const CONFIG_SERVER_PORT: &KvsConstKey<u16> = &KvsConstKey::new(CONFIG_NAMESPACE, "server_port");

const CONFIG_AUTO_SYNC_DELAY: &KvsConstKey<u64> =
    &KvsConstKey::new(CONFIG_NAMESPACE, "auto_sync_delay_in_seconds");

const CONFIG_AUTO_COMMIT_DELAY: &KvsConstKey<u64> =
    &KvsConstKey::new(CONFIG_NAMESPACE, "auto_commit_delay_in_seconds");

pub trait ArhivConfigExt {
    fn get_server_port(&self) -> Result<u16>;

    fn get_auto_sync_delay(&self) -> Result<Duration>;

    fn get_auto_commit_delay(&self) -> Result<Duration>;
}

impl ArhivConfigExt for BazaConnection {
    fn get_server_port(&self) -> Result<u16> {
        let port = self
            .kvs_const_get(CONFIG_SERVER_PORT)?
            .unwrap_or(DEFAULT_SERVER_PORT);

        Ok(port)
    }

    fn get_auto_sync_delay(&self) -> Result<Duration> {
        let delay = self
            .kvs_const_get(CONFIG_AUTO_SYNC_DELAY)?
            .unwrap_or(DEFAULT_AUTO_SYNC_DELAY_IN_SECONDS);

        Ok(Duration::from_secs(delay))
    }

    fn get_auto_commit_delay(&self) -> Result<Duration> {
        let delay = self
            .kvs_const_get(CONFIG_AUTO_COMMIT_DELAY)?
            .unwrap_or(DEFAULT_AUTO_COMMIT_DELAY_IN_SECONDS);

        Ok(Duration::from_secs(delay))
    }
}

#[derive(Debug)]
pub struct Config {
    pub server_port: u16,

    pub auto_commit_delay_in_seconds: u64,

    pub auto_sync_delay_in_seconds: u64,
}

impl Arhiv {
    pub fn get_config(&self) -> Result<Config> {
        let conn = self.baza.get_connection()?;

        Ok(Config {
            server_port: conn.get_server_port()?,
            auto_commit_delay_in_seconds: conn.get_auto_commit_delay()?.as_secs(),
            auto_sync_delay_in_seconds: conn.get_auto_sync_delay()?.as_secs(),
        })
    }
}
