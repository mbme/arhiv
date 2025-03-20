use std::fmt::Display;

use anyhow::{Context, Result};
use keyring::Entry;

use crate::{log, ExposeSecret, SecretString};

pub trait Keyring: Send + Sync {
    fn get_password(&self) -> Result<Option<SecretString>>;

    fn set_password(&self, password: Option<SecretString>) -> Result<()>;
}

/// Noop keyring implementation, primarily for development & tests.
pub struct NoopKeyring;

impl Keyring for NoopKeyring {
    fn get_password(&self) -> Result<Option<SecretString>> {
        Ok(None)
    }

    fn set_password(&self, _password: Option<SecretString>) -> Result<()> {
        Ok(())
    }
}

/// Keyring implementation that relies on system keyring.
/// Works on Windows, Linux, Mac & iOS.
pub struct SystemKeyring {
    service: String,
    user: String,
}

impl SystemKeyring {
    pub fn new(service: impl Into<String>, user: impl Into<String>) -> Self {
        SystemKeyring {
            service: service.into(),
            user: user.into(),
        }
    }

    fn new_entry(&self) -> Entry {
        Entry::new(&self.service, &self.user).expect("Failed to create keyring Entry")
    }
}

impl Display for SystemKeyring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Keyring {}/{}]", self.service, self.user)
    }
}

impl Keyring for SystemKeyring {
    fn get_password(&self) -> Result<Option<SecretString>> {
        log::info!("{self}: Reading password");

        let entry = self.new_entry();

        let password = match entry.get_password() {
            Ok(password) => password,
            Err(keyring::Error::NoEntry) => {
                log::info!("{self}: Couldn't find password");
                return Ok(None);
            }
            Err(err) => {
                log::warn!("{self}: Failed to retrieve password: {err}");

                return Err(err.into());
            }
        };

        let password: SecretString = password.into();

        Ok(Some(password))
    }

    fn set_password(&self, password: Option<SecretString>) -> Result<()> {
        if let Some(password) = password {
            log::info!("{self}: Saving password");

            let entry = self.new_entry();

            entry
                .set_password(password.expose_secret())
                .context("Failed to save password to keyring")?;
        } else {
            log::info!("{self}: Erasing password");

            let entry = self.new_entry();

            match entry.delete_credential() {
                Ok(_) => {}
                Err(keyring::Error::NoEntry) => {
                    log::info!("{self}: Erasing password: there was no password in keyring");
                }
                Err(err) => {
                    log::warn!("{self}: Failed to erase password from keyring: {err}");

                    return Err(err.into());
                }
            };
        }

        Ok(())
    }
}
