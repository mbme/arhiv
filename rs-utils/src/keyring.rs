use std::fmt::Display;

use anyhow::{Context, Result};
use keyring::Entry;

use crate::{log, ExposeSecret, SecretString};

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

        let entry = self.new_entry(name);

        let value = match entry.get_password() {
            Ok(value) => value,
            Err(keyring::Error::NoEntry) => {
                log::info!("{self}: Couldn't find {name}");
                return Ok(None);
            }
            Err(err) => {
                log::warn!("{self}: Failed to retrieve {name}: {err}");

                return Err(err.into());
            }
        };

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
                Err(keyring::Error::NoEntry) => {
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
