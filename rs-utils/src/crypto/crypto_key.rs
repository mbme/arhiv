use anyhow::{anyhow, ensure, Result};
use argon2::{Argon2, Params};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::new_random_crypto_byte_array;

use super::SecretBytes;

type HmacSha256 = Hmac<Sha256>;

pub const KEY_SIZE: usize = 32;
pub const SALT_SIZE: usize = 32;

pub type Salt = [u8; SALT_SIZE];
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

    pub fn random_salt() -> Salt {
        new_random_crypto_byte_array()
    }

    pub fn salt_from_data(salt_material: impl AsRef<[u8]>) -> Result<Salt> {
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

    pub fn derive_subkey(crypto_material: &[u8], salt: Salt) -> Result<Self> {
        ensure!(
            crypto_material.len() >= Self::MIN_SUBKEY_CRYPTO_MATERIAL_LEN,
            "Crypto key must be at least {} bytes long, got {} instead",
            Self::MIN_SUBKEY_CRYPTO_MATERIAL_LEN,
            crypto_material.len()
        );

        // HKDF: derive subkey using crypto hash of the cryptographic key material & salt
        let key = blake3::keyed_hash(&salt, crypto_material).into();

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

    pub fn sign(&self, msg: &[u8]) -> [u8; 32] {
        let mut mac = HmacSha256::new_from_slice(&self.key).expect("HMAC can take key of any size");
        mac.update(msg);

        let result = mac.finalize();

        result.into_bytes().into()
    }

    pub fn verify_signature(&self, msg: &[u8], tag: &[u8]) -> bool {
        let mut mac = HmacSha256::new_from_slice(&self.key).expect("HMAC can take key of any size");

        mac.update(msg);

        mac.verify_slice(tag).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_key() -> Result<()> {
        let password = "12345678".into();
        let salt = CryptoKey::random_salt();
        let key1 = CryptoKey::derive_from_password_with_argon2(&password, salt)?;
        let key2 = CryptoKey::derive_from_password_with_argon2(&password, salt)?;

        assert_eq!(key1.get(), key2.get());

        Ok(())
    }

    #[test]
    fn test_hmac() -> Result<()> {
        let key =
            CryptoKey::derive_subkey([0; 32].as_slice(), CryptoKey::salt_from_data("test1234")?)?;

        let msg1 = b"message1";
        let tag1 = key.sign(msg1);

        let msg2 = b"message2";
        let tag2 = key.sign(msg2);

        assert!(key.verify_signature(msg1, &tag1));
        assert!(key.verify_signature(msg2, &tag2));

        assert!(!key.verify_signature(msg2, &tag1));
        assert!(!key.verify_signature(msg1, &tag2));

        Ok(())
    }
}
