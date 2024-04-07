use std::time::Duration;

use anyhow::Result;

use baza::{BazaConnection, KvsConstKey};

const DEFAULT_AUTO_COMMIT_DELAY_IN_SECONDS: u64 = 600;
const DEFAULT_AUTO_SYNC_DELAY_IN_SECONDS: u64 = 20;

const CONFIG_NAMESPACE: &str = "arhiv-config";

const CONFIG_AUTO_SYNC_DELAY: &KvsConstKey<u64> =
    &KvsConstKey::new(CONFIG_NAMESPACE, "auto_sync_delay_in_seconds");

const CONFIG_AUTO_COMMIT_DELAY: &KvsConstKey<u64> =
    &KvsConstKey::new(CONFIG_NAMESPACE, "auto_commit_delay_in_seconds");

pub trait ArhivConfigExt {
    fn get_auto_sync_delay(&self) -> Result<Duration>;
    fn set_auto_sync_delay(&self, delay_in_seconds: u64) -> Result<()>;

    fn get_auto_commit_delay(&self) -> Result<Duration>;
    fn set_auto_commit_delay(&self, delay_in_seconds: u64) -> Result<()>;
}

impl ArhivConfigExt for BazaConnection {
    fn get_auto_sync_delay(&self) -> Result<Duration> {
        let delay = self
            .kvs_const_get(CONFIG_AUTO_SYNC_DELAY)?
            .unwrap_or(DEFAULT_AUTO_SYNC_DELAY_IN_SECONDS);

        Ok(Duration::from_secs(delay))
    }

    fn set_auto_sync_delay(&self, delay_in_seconds: u64) -> Result<()> {
        self.kvs_const_set(CONFIG_AUTO_SYNC_DELAY, &delay_in_seconds)
    }

    fn get_auto_commit_delay(&self) -> Result<Duration> {
        let delay = self
            .kvs_const_get(CONFIG_AUTO_COMMIT_DELAY)?
            .unwrap_or(DEFAULT_AUTO_COMMIT_DELAY_IN_SECONDS);

        Ok(Duration::from_secs(delay))
    }

    fn set_auto_commit_delay(&self, delay_in_seconds: u64) -> Result<()> {
        self.kvs_const_set(CONFIG_AUTO_COMMIT_DELAY, &delay_in_seconds)
    }
}
