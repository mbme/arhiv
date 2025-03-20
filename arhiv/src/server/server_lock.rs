use std::fs;

use anyhow::{anyhow, Context, Result};

use rs_utils::{bytes_to_hex_string, hex_string_to_bytes, log, LockFile, Token};

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

    pub fn read_server_info(&self) -> Result<(u16, Token)> {
        let data = fs::read_to_string(&self.lock_file)?;

        let (port, token) = data
            .trim()
            .split_once(' ')
            .context("Failed to split data from server lock file")?;

        let port = port.parse::<u16>().context("Failed to parse port as u16")?;

        let token = hex_string_to_bytes(token).context("Failed to parse token as hex string")?;
        let token: Token = token
            .try_into()
            .map_err(|err| anyhow!("Failed to parse token as byte array: {err:?}"))?;

        Ok((port, token))
    }

    pub fn write_server_info(&self, port: u16, token: &Token) -> Result<()> {
        let data = format!("{port} {}", bytes_to_hex_string(token));

        fs::write(&self.lock_file, data).context("Failed to write server info into lock file")
    }
}
