use anyhow::{Context, Result, bail};
use dialoguer::{Password, theme::ColorfulTheme};

use arhiv::{Arhiv, CacheUnlockResult};
use baza::BazaManager;
use baza_common::{SecretString, log};

pub(crate) fn unlock_arhiv(arhiv: &Arhiv) -> Result<()> {
    if !arhiv.baza.storage_exists()? {
        bail!("Arhiv not initialized");
    }

    if !arhiv.baza.key_exists()? {
        bail!("Arhiv key is missing. First need to import key.");
    }

    match arhiv.unlock_using_keyring() {
        Ok(CacheUnlockResult::Unlocked) => return Ok(()),
        Ok(CacheUnlockResult::NeedsPassword) => {
            log::debug!("No usable cached storage key");
        }
        Err(err) => {
            log::error!("Failed to use keyring: {err}");
        }
    }

    println!("Please enter password");
    let password = prompt_password(BazaManager::MIN_PASSWORD_LENGTH, false)?;

    arhiv.unlock(password)?;
    Ok(())
}

pub(crate) fn unlocked_desktop_arhiv() -> Result<Arhiv> {
    let arhiv = Arhiv::new_desktop();
    unlock_arhiv(&arhiv)?;
    Ok(arhiv)
}

pub(crate) fn prompt_password(min_length: usize, with_confirmation: bool) -> Result<SecretString> {
    let theme = ColorfulTheme::default();

    let mut input =
        Password::with_theme(&theme).with_prompt(format!("Password (min {min_length} symbols):"));

    if with_confirmation {
        input = input.with_confirmation("Repeat password", "Error: the passwords don't match.");
    }

    input = input.validate_with(|input: &String| -> Result<(), String> {
        if input.chars().count() >= min_length {
            Ok(())
        } else {
            Err(format!("Password must be longer than {min_length}"))
        }
    });

    input
        .interact()
        .map(|value| value.into())
        .context("Failed to prompt password")
}
