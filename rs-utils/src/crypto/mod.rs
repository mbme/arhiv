use std::{
    fs::File,
    io::{BufReader, Read},
};

use anyhow::{ensure, Context, Result};
use data_encoding::{BASE64, BASE64URL, HEXUPPER};

mod auth_token;
mod certificate;
mod secret;
pub use auth_token::*;
pub use certificate::*;
pub use secret::*;

pub fn get_file_hash_blake3(file_path: &str) -> Result<Vec<u8>> {
    let mut hasher = blake3::Hasher::new();

    let file = File::open(file_path).context("failed to open file")?;
    let mut reader = BufReader::new(file);
    let mut buffer = [0; 1024 * 1024]; // 1Mb cache

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }

        hasher.update(&buffer[..count]);
    }

    let hash = hasher.finalize();

    Ok(hash.as_bytes().to_vec())
}

#[must_use]
pub fn get_string_hash_blake3(data: &str) -> String {
    let mut hasher = blake3::Hasher::new();

    hasher.update(data.as_bytes());

    let hash = hasher.finalize();

    hash.to_string()
}

#[must_use]
pub fn to_url_safe_base64(bytes: &[u8]) -> String {
    BASE64URL.encode(bytes)
}

#[must_use]
pub fn is_valid_base64(value: &str) -> bool {
    BASE64URL.decode(value.as_bytes()).is_ok()
}

pub fn decode_base64(data: &str) -> Result<Vec<u8>> {
    BASE64
        .decode(data.as_bytes())
        .context("Failed to decode base64 string")
}

pub fn bytes_to_hex_string(bytes: &[u8]) -> String {
    HEXUPPER.encode(bytes)
}

pub fn hex_string_to_bytes(hex: &str) -> Result<Vec<u8>> {
    HEXUPPER
        .decode(hex.as_bytes())
        .context("Failed to decode hex string")
}

// Derive secure key from password & salt
pub struct PBKDF {
    key: SecretBytes,
}

impl PBKDF {
    pub const MIN_PASSWORD_LENGTH: usize = 8;
    pub const MIN_SALT_LENGTH: usize = 8;

    pub fn derive(password: &[u8], salt: &str) -> Result<Self> {
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

        Ok(Self::generate(password, salt))
    }

    fn generate(password: &[u8], salt: &str) -> Self {
        let key = SecretBytes::new(blake3::derive_key(salt, password).into());

        Self { key }
    }

    pub fn get(&self) -> &SecretBytes {
        &self.key
    }
}

// Sign & verify data
pub struct HMAC {
    key: PBKDF,
}

impl HMAC {
    pub fn new_from_password(password: impl AsRef<[u8]>, salt: impl AsRef<str>) -> Result<Self> {
        let key = PBKDF::derive(password.as_ref(), salt.as_ref())?;

        Ok(Self { key })
    }

    pub fn new(key: PBKDF) -> Self {
        Self { key }
    }

    pub fn sign(&self, msg: &[u8]) -> [u8; 32] {
        let key = self
            .key
            .get()
            .as_bytes()
            .try_into()
            .expect("key must have correct length");

        let hash = blake3::keyed_hash(key, msg);

        hash.into()
    }

    pub fn verify(&self, msg: &[u8], tag: &[u8]) -> bool {
        let original_hash = if let Ok(tag) = TryInto::<&[u8; 32]>::try_into(tag) {
            blake3::Hash::from_bytes(*tag)
        } else {
            return false;
        };

        let hash = blake3::Hash::from_bytes(self.sign(msg));

        hash == original_hash
    }
}

#[cfg(test)]
mod tests {
    use rand::RngCore;

    use super::*;
    use crate::workspace_relpath;

    #[test]
    fn test_get_file_hash_blake3() -> Result<()> {
        let src = &workspace_relpath("resources/k2.jpg");

        assert_eq!(
            bytes_to_hex_string(&get_file_hash_blake3(src)?),
            "33853BF0E88A13956014F28000EA7E6A8D362178E79ADAF3098F3F0B29D60301"
        );

        Ok(())
    }

    #[test]
    fn test_get_string_hash_blake3() {
        assert_eq!(
            get_string_hash_blake3("test"),
            "4878ca0425c739fa427f7eda20fe845f6b2e46ba5fe2a14df5b1e32f50603215"
        );
    }

    #[test]
    fn test_hex_encode_decode() {
        let mut data = [0u8; 150];
        rand::thread_rng().fill_bytes(&mut data);

        let result = bytes_to_hex_string(&data);
        let result = hex_string_to_bytes(&result).unwrap();

        assert_eq!(data, result.as_slice());
    }

    #[test]
    fn test_auth_token_parse_serialize() {
        let hmac = HMAC::new_from_password("test1234", "test1234").unwrap();

        let token = AuthToken::generate(&hmac);

        let token_str = token.serialize();

        let parsed_token = AuthToken::parse(&token_str).unwrap();

        assert_eq!(parsed_token, token);
    }
}
