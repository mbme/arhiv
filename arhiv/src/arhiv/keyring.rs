use std::sync::Arc;
use std::{fmt::Display, time::Instant};

use anyhow::{Context, Result};
use keyring::{Entry, Error};

use baza::DEV_MODE;
use rs_utils::{ExposeSecret, SecretString, log};

pub trait Keyring {
    fn get_string(&self, name: &str) -> Result<Option<SecretString>>;

    fn set_string(&self, name: &str, value: Option<SecretString>) -> Result<()>;
}

/// Noop keyring implementation, primarily for development & tests.
pub struct NoopKeyring;

impl Keyring for NoopKeyring {
    fn get_string(&self, _name: &str) -> Result<Option<SecretString>> {
        Ok(None)
    }

    fn set_string(&self, _name: &str, _value: Option<SecretString>) -> Result<()> {
        Ok(())
    }
}

/// Keyring implementation that relies on system keyring.
/// Works on Windows, Linux, Mac & iOS.
pub struct SystemKeyring {
    service: String,
}

impl SystemKeyring {
    pub fn new(service: impl Into<String>) -> Self {
        SystemKeyring {
            service: service.into(),
        }
    }

    fn new_entry(&self, name: &str) -> Entry {
        Entry::new(&self.service, name).expect("Failed to create keyring Entry")
    }
}

impl Display for SystemKeyring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Keyring {}]", self.service)
    }
}

impl Keyring for SystemKeyring {
    fn get_string(&self, name: &str) -> Result<Option<SecretString>> {
        log::info!("{self}: Reading {name}");

        let start_time = Instant::now();

        let entry = self.new_entry(name);

        let value = match entry.get_password() {
            Ok(value) => value,
            Err(Error::NoEntry) => {
                log::info!("{self}: Couldn't find {name}");
                return Ok(None);
            }
            Err(err) => {
                log::warn!("{self}: Failed to retrieve {name}: {err}");

                return Err(err.into());
            }
        };

        let duration = start_time.elapsed();
        log::debug!("{self}: Read {name} in {duration:?}");

        let value: SecretString = value.into();

        Ok(Some(value))
    }

    fn set_string(&self, name: &str, value: Option<SecretString>) -> Result<()> {
        if let Some(value) = value {
            log::info!("{self}: Saving {name}");

            let entry = self.new_entry(name);

            entry
                .set_password(value.expose_secret())
                .context("Failed to save {name} to keyring")?;
        } else {
            log::info!("{self}: Erasing {name}");

            let entry = self.new_entry(name);

            match entry.delete_credential() {
                Ok(_) => {}
                Err(Error::NoEntry) => {
                    log::info!("{self}: Erasing {name}: there was no {name} in keyring");
                }
                Err(err) => {
                    log::warn!("{self}: Failed to erase {name} from keyring: {err}");

                    return Err(err.into());
                }
            };
        }

        Ok(())
    }
}

pub struct ArhivKeyring {
    keyring: Arc<dyn Keyring + Send + Sync>,
}

impl ArhivKeyring {
    pub const PASSWORD: &str = "password";

    pub fn new(keyring: Arc<dyn Keyring + Send + Sync>) -> ArhivKeyring {
        ArhivKeyring { keyring }
    }

    pub fn new_noop() -> ArhivKeyring {
        ArhivKeyring::new(Arc::new(NoopKeyring))
    }

    pub fn new_system_keyring() -> ArhivKeyring {
        ArhivKeyring::new(Arc::new(SystemKeyring::new(if DEV_MODE {
            "Arhiv-dev"
        } else {
            "Arhiv"
        })))
    }

    pub fn get_password(&self) -> Result<Option<SecretString>> {
        self.keyring.get_string(ArhivKeyring::PASSWORD)
    }

    pub fn set_password(&self, password: Option<SecretString>) -> Result<()> {
        self.keyring.set_string(ArhivKeyring::PASSWORD, password)
    }
}
