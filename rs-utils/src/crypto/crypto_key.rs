use anyhow::{anyhow, ensure, Result};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, SecretString};
use sha2::{Digest, Sha256};

use super::SecretByteArray;

type HmacSha256 = Hmac<Sha256>;
type HkdfSha256 = Hkdf<Sha256>;

pub const KEY_SIZE: usize = 32;
pub const SALT_SIZE: usize = 32;
pub const SIGNATURE_SIZE: usize = 32;

pub type Key = SecretByteArray<KEY_SIZE>;
pub type Salt = SecretByteArray<SALT_SIZE>;

pub type Signature = [u8; SIGNATURE_SIZE];

/// Derive secure key from a password & salt.
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

    pub fn new_random_key() -> Self {
        Self::new(Key::new_random(), Salt::new_random())
    }

    pub fn salt_from_data(salt_material: impl AsRef<[u8]>) -> Result<Salt> {
        let salt_material = salt_material.as_ref();
        ensure!(
            salt_material.len() >= Self::MIN_SALT_MATERIAL_LENGTH,
            "Salt material must be at least {}b, got {}b",
            Self::MIN_SALT_MATERIAL_LENGTH,
            salt_material.len()
        );

        let mut hasher = Sha256::new();

        hasher.update(salt_material);

        let hash = hasher.finalize();

        Ok(Salt::new(hash.into()))
    }

    pub fn derive_from_password_with_scrypt(password: &SecretString, salt: Salt) -> Result<Self> {
        ensure!(
            password.expose_secret().len() >= Self::MIN_PASSWORD_LEN,
            "password must consist of at least {} bytes",
            Self::MIN_PASSWORD_LEN,
        );

        #[cfg(test)]
        let log_n = 1;
        #[cfg(not(test))]
        let log_n = 15;
        let params = scrypt::Params::new(log_n, 8, 1, 32).expect("Scrypt params are invalid");

        let mut output = [0u8; KEY_SIZE];
        scrypt::scrypt(
            password.expose_secret().as_bytes(),
            salt.expose_secret(),
            &params,
            &mut output,
        )
        .expect("output is the correct length");

        Ok(CryptoKey {
            key: Key::new(output),
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
        let hkdf = HkdfSha256::new(Some(salt.expose_secret()), crypto_material);
        let mut key = [0u8; KEY_SIZE];
        hkdf.expand(&[], &mut key)
            .map_err(|err| anyhow!("Key derivation failed: {err}"))?;

        Ok(Self {
            key: Key::new(key),
            salt,
        })
    }

    #[must_use]
    pub fn get(&self) -> &Key {
        &self.key
    }

    #[must_use]
    pub fn get_salt(&self) -> &Salt {
        &self.salt
    }

    pub fn sign(&self, msg: &[u8]) -> Signature {
        let mut mac = HmacSha256::new_from_slice(self.key.expose_secret())
            .expect("HMAC can take key of any size");
        mac.update(msg);

        let result = mac.finalize();

        result.into_bytes().into()
    }

    pub fn verify_signature(&self, msg: &[u8], tag: &Signature) -> bool {
        let mut mac = HmacSha256::new_from_slice(self.key.expose_secret())
            .expect("HMAC can take key of any size");

        mac.update(msg);

        mac.verify_slice(tag).is_ok()
    }
}

impl Clone for CryptoKey {
    fn clone(&self) -> Self {
        Self::new(
            Key::new(*self.key.expose_secret()),
            Salt::new(*self.salt.expose_secret()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_key() -> Result<()> {
        let password: SecretString = "12345678".into();
        let salt = Salt::new_random();
        let key1 = CryptoKey::derive_from_password_with_scrypt(&password, salt.clone())?;
        let key2 = CryptoKey::derive_from_password_with_scrypt(&password, salt)?;

        assert_eq!(key1.get().expose_secret(), key2.get().expose_secret());

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
