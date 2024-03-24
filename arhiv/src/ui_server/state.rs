use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Context, Result};

use rs_utils::SelfSignedCertificate;

use crate::{Arhiv, ArhivOptions};

pub struct UIState {
    options: ArhivOptions,
    arhiv: RwLock<Option<Arc<Arhiv>>>,
}

impl UIState {
    pub fn new(root_dir: &str, mut options: ArhivOptions) -> Result<Self> {
        let certificate = options
            .certificate
            .unwrap_or_else(Arhiv::generate_certificate);
        options.certificate = Some(certificate.clone());

        let arhiv = if Arhiv::exists(root_dir) {
            let arhiv = Arhiv::open(root_dir, options.clone())?;
            let arhiv = Arc::new(arhiv);

            Some(arhiv)
        } else {
            None
        };
        let arhiv = RwLock::new(arhiv);

        Ok(Self { options, arhiv })
    }

    pub fn must_get_arhiv(&self) -> Result<Arc<Arhiv>> {
        self.arhiv
            .read()
            .map_err(|err| anyhow!("Failed to acquire read lock UIState.arhiv: {err}"))?
            .clone()
            .context("UIState.arhiv is None")
    }

    #[must_use]
    pub fn get_certificate(&self) -> &SelfSignedCertificate {
        self.options
            .certificate
            .as_ref()
            .expect("Certificate must be available")
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
}
