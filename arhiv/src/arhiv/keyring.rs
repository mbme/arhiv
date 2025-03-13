use anyhow::{Context, Result};
use keyring::Entry;

use baza::DEV_MODE;
use rs_utils::{log, ExposeSecret, SecretString};

use super::Arhiv;

impl Arhiv {
    fn new_entry() -> Entry {
        Entry::new(if DEV_MODE { "Arhiv-dev" } else { "Arhiv" }, "Arhiv")
            .expect("Failed to create keyring Entry")
    }

    pub fn unlock_using_keyring(&self) -> Result<bool> {
        log::debug!("Unlocking Arhiv using password from keyring");

        let entry = Arhiv::new_entry();

        let password = match entry.get_password() {
            Ok(password) => password,
            Err(keyring::Error::NoEntry) => {
                log::debug!("Couldn't find Arhiv password in keyring");
                return Ok(false);
            }
            Err(err) => {
                log::warn!("Failed to retrieve password from keyring: {err}");

                return Err(err.into());
            }
        };
        let password: SecretString = password.into();

        self.baza
            .unlock(password)
            .context("Failed to unlock Arhiv using password from keyring")?;

        Ok(true)
    }

    pub fn is_password_in_keyring(&self) -> Result<bool> {
        let entry = Arhiv::new_entry();

        match entry.get_password() {
            Ok(_) => Ok(true),
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(err) => {
                log::warn!("Failed to retrieve password from keyring: {err}");

                return Err(err.into());
            }
        }
    }

    pub fn save_password_to_keyring(password: SecretString) -> Result<()> {
        log::debug!("Saving Arhiv password to keyring");

        let entry = Arhiv::new_entry();

        entry
            .set_password(password.expose_secret())
            .context("Failed to save Arhiv password to keyring")
    }

    pub fn erase_password_from_keyring() -> Result<()> {
        log::debug!("Erasing Arhiv password from keyring");

        let entry = Arhiv::new_entry();

        match entry.delete_credential() {
            Ok(_) => {}
            Err(keyring::Error::NoEntry) => {
                log::debug!("Erasing Arhiv password: there was no password in keyring");
            }
            Err(err) => {
                log::warn!("Failed to erase password from keyring: {err}");

                return Err(err.into());
            }
        };

        Ok(())
    }
}
