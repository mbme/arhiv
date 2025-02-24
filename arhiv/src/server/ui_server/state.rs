use std::sync::{Arc, RwLock};

use anyhow::{anyhow, bail, Context, Result};

use baza::{sync::create_shared_key, Credentials};
use rs_utils::crypto_key::CryptoKey;

use crate::{Arhiv, ArhivOptions};

pub struct UIState {
    root_dir: String,
    options: ArhivOptions,
    arhiv: RwLock<Option<Arc<Arhiv>>>,
    server_port: RwLock<Option<u16>>,
    shared_key: RwLock<Option<Arc<CryptoKey>>>,
}

impl UIState {
    pub fn new(root_dir: &str, options: ArhivOptions) -> Result<Self> {
        let mut shared_key = None;
        let arhiv = if Arhiv::exists(root_dir) {
            let arhiv = Arhiv::open(root_dir, options.clone())?;
            let arhiv = Arc::new(arhiv);

            shared_key = Some(Arc::new(create_shared_key(&arhiv.baza)?));

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
            shared_key: RwLock::new(shared_key),
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

    pub fn must_get_shared_key(&self) -> Result<Arc<CryptoKey>> {
        self.shared_key
            .read()
            .map_err(|err| anyhow!("Failed to acquire read lock UIState.shared_key: {err}"))?
            .clone()
            .context("UIState.shared_key is None")
    }

    pub fn create_arhiv(&self, auth: Credentials) -> Result<()> {
        let mut lock_guard = self
            .arhiv
            .write()
            .map_err(|err| anyhow!("Failed to acquire write lock UIState.arhiv: {err}"))?;

        if lock_guard.is_some() {
            bail!("Arhiv already exists");
        }

        Arhiv::create(&self.root_dir, auth)?;

        let arhiv = Arhiv::open(&self.root_dir, self.options.clone())?;
        let arhiv = Arc::new(arhiv);

        let server_port = self
            .server_port
            .read()
            .map_err(|err| anyhow!("Failed to acquired read lock UIState.server_port: {err}"))?;

        self.shared_key
            .write()
            .map_err(|err| anyhow!("Failed to acquire write lock UIState.shared_key: {err}"))?
            .replace(Arc::new(create_shared_key(&arhiv.baza)?));

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
