use anyhow::{anyhow, ensure, Result};
use argon2::{Argon2, Params, MIN_SALT_LEN};

use super::SecretBytes;

/// Derive secure key from a password & salt.
pub struct CryptoKey {
    key: SecretBytes,
    salt: Vec<u8>,
}

impl CryptoKey {
    pub const MIN_PASSWORD_LENGTH: usize = 8;
    pub const MIN_SALT_LENGTH: usize = MIN_SALT_LEN;

    const DERIVED_KEY_SIZE_IN_BYTES: usize = 32;

    /// Argon2id v19 (m=19456 (19 MiB), t=2, p=1)
    /// https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#argon2id
    pub fn derive_from_password_with_argon2(
        password: &SecretBytes,
        salt: impl Into<Vec<u8>>,
    ) -> Result<Self> {
        let salt = salt.into();

        ensure!(
            password.len() >= Self::MIN_PASSWORD_LENGTH,
            "password must consist of at least {} bytes",
            Self::MIN_PASSWORD_LENGTH,
        );

        ensure!(
            salt.len() >= Self::MIN_SALT_LENGTH,
            "salt must consist of at least {} bytes",
            Self::MIN_SALT_LENGTH
        );

        let params = Params::new(19456, 2, 1, Some(Self::DERIVED_KEY_SIZE_IN_BYTES))
            .map_err(|err| anyhow!("Failed to construct Argon2 params: {err}"))?;

        let mut output_key_material = [0u8; Self::DERIVED_KEY_SIZE_IN_BYTES];
        Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
            .hash_password_into(password.as_bytes(), &salt, &mut output_key_material)
            .map_err(|err| anyhow!("Failed to derive CryptoKey from password: {err}"))?;

        Ok(CryptoKey {
            key: SecretBytes::new(output_key_material.into()),
            salt,
        })
    }

    pub fn from_crypto_bytes(key: impl Into<SecretBytes>, salt: Option<Vec<u8>>) -> Result<Self> {
        let key = key.into();

        ensure!(
            key.len() >= Self::DERIVED_KEY_SIZE_IN_BYTES,
            "Crypto key must be at least {} bytes long, got {} instead",
            Self::DERIVED_KEY_SIZE_IN_BYTES,
            key.len()
        );

        Ok(Self {
            key,
            salt: salt.unwrap_or_default(),
        })
    }

    pub fn derive_subkey(&self, salt: &str) -> Result<Self> {
        ensure!(
            salt.len() >= Self::MIN_SALT_LENGTH,
            "salt must consist of at least {} bytes",
            Self::MIN_SALT_LENGTH
        );

        let subkey = blake3::derive_key(salt, self.key.as_bytes()).into();
        let subkey = SecretBytes::new(subkey);

        Ok(Self {
            key: subkey,
            salt: salt.into(),
        })
    }

    #[must_use]
    pub fn get(&self) -> &SecretBytes {
        &self.key
    }

    #[must_use]
    pub fn get_salt(&self) -> &[u8] {
        &self.salt
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.key.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.key.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_key() -> Result<()> {
        let password = "12345678".into();
        let key1 = CryptoKey::derive_from_password_with_argon2(&password, "salt1234")?;
        let key2 = CryptoKey::derive_from_password_with_argon2(&password, "salt1234")?;

        assert_eq!(key1.get().as_bytes(), key2.get().as_bytes());

        Ok(())
    }

    #[test]
    fn test_derive_subkey() -> Result<()> {
        {
            let key1 = CryptoKey::from_crypto_bytes([1; 32].as_slice(), None)?
                .derive_subkey("salt1234")?;
            let key2 = CryptoKey::from_crypto_bytes([1; 32].as_slice(), None)?
                .derive_subkey("salt1234")?;

            assert_eq!(key1.get().as_bytes(), key2.get().as_bytes());
        }

        {
            let key1 = CryptoKey::from_crypto_bytes([1; 32].as_slice(), None)?
                .derive_subkey("salt1234")?;
            let key2 = CryptoKey::from_crypto_bytes([1; 32].as_slice(), None)?
                .derive_subkey("salt12345")?;

            assert_ne!(key1.get().as_bytes(), key2.get().as_bytes());
        }

        Ok(())
    }
}
