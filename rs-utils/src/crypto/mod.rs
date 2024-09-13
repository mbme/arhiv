use std::{
    fs::File,
    io::{BufReader, Read},
    num::NonZeroU32,
};

use anyhow::{ensure, Context, Result};
use data_encoding::{BASE64, BASE64URL, HEXUPPER};
use ring::{
    digest,
    hmac::{self, HMAC_SHA256},
    pbkdf2,
};

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

type PBKDF2Key = [u8; digest::SHA512_256_OUTPUT_LEN];

// Derive secure key from password & salt
pub struct PBKDF2 {
    key: PBKDF2Key,
}

impl PBKDF2 {
    pub const MIN_PASSWORD_LENGTH: usize = 8;
    pub const MIN_SALT_LENGTH: usize = 8;

    pub fn derive(password: &[u8], salt: &[u8]) -> Result<Self> {
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

        let key = Self::generate(password, salt);

        Ok(Self { key })
    }

    fn generate(password: &[u8], salt: &[u8]) -> PBKDF2Key {
        let mut key = [0u8; digest::SHA512_256_OUTPUT_LEN];

        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(100_000).expect("iterations count must be non-zero"),
            salt,
            password,
            &mut key,
        );

        key
    }

    pub fn get(&self) -> &PBKDF2Key {
        &self.key
    }
}

// Sign & verify data
#[derive(Debug)]
pub struct HMAC {
    key: hmac::Key,
}

impl HMAC {
    pub fn new_from_password(password: impl AsRef<[u8]>, salt: impl AsRef<[u8]>) -> Result<Self> {
        let key = PBKDF2::derive(password.as_ref(), salt.as_ref())?;

        Ok(Self::new(&key))
    }

    pub fn new(key: &PBKDF2) -> Self {
        let key = hmac::Key::new(HMAC_SHA256, key.get());

        Self { key }
    }

    pub fn sign(&self, msg: &[u8]) -> Vec<u8> {
        let mut context = hmac::Context::with_key(&self.key);
        context.update(msg);

        let tag = context.sign();

        tag.as_ref().to_vec()
    }

    pub fn verify(&self, msg: &[u8], tag: &[u8]) -> bool {
        hmac::verify(&self.key, msg, tag).is_ok()
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
