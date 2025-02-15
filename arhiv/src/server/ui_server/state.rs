use std::sync::{Arc, RwLock};

use anyhow::{anyhow, bail, Context, Result};
use rs_utils::SecretString;

use crate::{Arhiv, ArhivOptions};

pub struct UIState {
    root_dir: String,
    options: ArhivOptions,
    arhiv: RwLock<Option<Arc<Arhiv>>>,
    server_port: RwLock<Option<u16>>,
}

impl UIState {
    pub fn new(root_dir: &str, options: ArhivOptions) -> Result<Self> {
        let arhiv = if Arhiv::exists(root_dir) {
            let arhiv = Arhiv::open(root_dir, options.clone())?;
            let arhiv = Arc::new(arhiv);

            Some(arhiv)
        } else {
            None
        };
        let arhiv = RwLock::new(arhiv);

        Ok(Self {
            root_dir: root_dir.to_string(),
            options,
            arhiv,
            server_port: Default::default(),
        })
    }

    pub fn arhiv_exists(&self) -> Result<bool> {
        let arhiv_exists = self
            .arhiv
            .read()
            .map_err(|err| anyhow!("Failed to acquire read lock UIState.arhiv: {err}"))?
            .is_some();

        Ok(arhiv_exists)
    }

    pub fn must_get_arhiv(&self) -> Result<Arc<Arhiv>> {
        self.arhiv
            .read()
            .map_err(|err| anyhow!("Failed to acquire read lock UIState.arhiv: {err}"))?
            .clone()
            .context("UIState.arhiv is None")
    }

    pub fn create_arhiv(&self, password: impl Into<SecretString>) -> Result<()> {
        let mut lock_guard = self
            .arhiv
            .write()
            .map_err(|err| anyhow!("Failed to acquire write lock UIState.arhiv: {err}"))?;

        if lock_guard.is_some() {
            bail!("Arhiv already exists");
        }

        Arhiv::create(&self.root_dir, password)?;

        let arhiv = Arhiv::open(&self.root_dir, self.options.clone())?;
        let arhiv = Arc::new(arhiv);

        let server_port = self
            .server_port
            .read()
            .map_err(|err| anyhow!("Failed to acquired read lock UIState.server_port: {err}"))?;

        if let Some(server_port) = *server_port {
            arhiv.start_mdns_server(server_port)?;
        }

        lock_guard.replace(arhiv);

        Ok(())
    }

    pub fn stop_arhiv(&self) -> Result<()> {
        if let Some(arhiv) = self
            .arhiv
            .write()
            .map_err(|err| anyhow!("Failed to acquired write lock UIState.arhiv: {err}"))?
            .take()
        {
            arhiv.stop();
        }

        Ok(())
    }

    pub fn start_mdns_server(&self, server_port: u16) -> Result<()> {
        self.server_port
            .write()
            .map_err(|err| anyhow!("Failed to acquired write lock UIState.server_port: {err}"))?
            .replace(server_port);

        let arhiv = self
            .arhiv
            .read()
            .map_err(|err| anyhow!("Failed to acquire read lock UIState.arhiv: {err}"))?;

        if let Some(ref arhiv) = *arhiv {
            arhiv.start_mdns_server(server_port)?;
        }

        Ok(())
    }
}
