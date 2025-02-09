use std::fs;

use anyhow::{Context, Result};

use rs_utils::{log, LockFile};

pub struct ArhivServerLock {
    lock_file: String,
    lock: Option<LockFile>,
}

impl ArhivServerLock {
    pub fn new(root_dir: &str) -> Self {
        let lock_file = format!("{root_dir}/arhiv-server.lock");
        log::debug!("Arhiv server lock file: {lock_file}");

        Self {
            lock_file,
            lock: None,
        }
    }

    pub fn acquire(&mut self) -> Result<()> {
        self.lock = Some(LockFile::must_lock(&self.lock_file)?);

        Ok(())
    }

    pub fn read_server_port(&self) -> Result<u16> {
        let value = fs::read_to_string(&self.lock_file)?;

        value
            .trim()
            .parse::<u16>()
            .with_context(|| format!("Can't parse port number from string `{}`", value))
    }

    pub fn write_server_port(&self, port: u16) -> Result<()> {
        fs::write(&self.lock_file, port.to_string())
            .context("Failed to write server port into lock file")
    }
}
