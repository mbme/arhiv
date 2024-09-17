use anyhow::{anyhow, ensure, Result};
use argon2::{Argon2, Params};
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::{new_random_byte_array, SecretBytes};

const KEY_SIZE: usize = 32;

pub type Salt = [u8; 32];
pub type Key = [u8; KEY_SIZE];

/// Derive secure key from a password & salt.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct CryptoKey {
    key: Key,
    salt: Salt,
}

impl CryptoKey {
    pub const MIN_PASSWORD_LEN: usize = 8;
    pub const MIN_SALT_MATERIAL_LENGTH: usize = 8;
    pub const MIN_SUBKEY_CRYPTO_MATERIAL_LEN: usize = 32;

    pub fn new(key: Key, salt: Salt) -> Self {
        Self { key, salt }
    }

    pub fn generate_salt() -> Salt {
        new_random_byte_array()
    }

    pub fn salt_from_string(salt_material: impl AsRef<[u8]>) -> Result<Salt> {
        let salt_material = salt_material.as_ref();
        ensure!(
            salt_material.len() >= Self::MIN_SALT_MATERIAL_LENGTH,
            "Salt material must be at least {}b, got {}b",
            Self::MIN_SALT_MATERIAL_LENGTH,
            salt_material.len()
        );

        Ok(blake3::hash(salt_material).into())
    }

    /// Argon2id v19 (m=19456 (19 MiB), t=2, p=1)
    /// https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html#argon2id
    pub fn derive_from_password_with_argon2(password: &SecretBytes, salt: Salt) -> Result<Self> {
        ensure!(
            password.len() >= Self::MIN_PASSWORD_LEN,
            "password must consist of at least {} bytes",
            Self::MIN_PASSWORD_LEN,
        );

        let params = Params::new(19456, 2, 1, Some(KEY_SIZE))
            .map_err(|err| anyhow!("Failed to construct Argon2 params: {err}"))?;

        let mut output_key_material = [0u8; KEY_SIZE];
        Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
            .hash_password_into(password.as_bytes(), &salt, &mut output_key_material)
            .map_err(|err| anyhow!("Failed to derive CryptoKey from password: {err}"))?;

        Ok(CryptoKey {
            key: output_key_material,
            salt,
        })
    }

    pub fn derive_subkey(crypto_material: impl Into<SecretBytes>, salt: &str) -> Result<Self> {
        let crypto_material = crypto_material.into();

        ensure!(
            crypto_material.len() >= Self::MIN_SUBKEY_CRYPTO_MATERIAL_LEN,
            "Crypto key must be at least {} bytes long, got {} instead",
            Self::MIN_SUBKEY_CRYPTO_MATERIAL_LEN,
            crypto_material.len()
        );

        let salt = Self::salt_from_string(salt)?;

        // HKDF: derive subkey using crypto hash of the cryptographic key material & salt
        let key = blake3::keyed_hash(&salt, crypto_material.as_bytes()).into();

        Ok(Self { key, salt })
    }

    #[must_use]
    pub fn get(&self) -> &Key {
        &self.key
    }

    #[must_use]
    pub fn get_salt(&self) -> &Salt {
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
        let salt = CryptoKey::generate_salt();
        let key1 = CryptoKey::derive_from_password_with_argon2(&password, salt)?;
        let key2 = CryptoKey::derive_from_password_with_argon2(&password, salt)?;

        assert_eq!(key1.get(), key2.get());

        Ok(())
    }
}
