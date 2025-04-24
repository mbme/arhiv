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

    pub fn read_server_info(&self) -> Result<u16> {
        let data = fs::read_to_string(&self.lock_file)?;

        let port = data
            .trim()
            .parse::<u16>()
            .context("Failed to parse port as u16")?;

        Ok(port)
    }

    pub fn write_server_info(&self, port: u16) -> Result<()> {
        fs::write(&self.lock_file, port.to_string())
            .context("Failed to write server info into lock file")
    }
}
