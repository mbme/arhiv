use std::{
    fs::File,
    io::{prelude::*, BufReader},
    num::NonZeroU32,
};

use anyhow::{ensure, Context, Result};
use data_encoding::{BASE64, BASE64URL, HEXUPPER};
use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType};
use ring::{
    digest,
    hmac::{self, HMAC_SHA256},
    pbkdf2,
};
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize)]
pub struct SelfSignedCertificate {
    pub private_key_der: Vec<u8>,
    pub certificate_der: Vec<u8>,
}

impl SelfSignedCertificate {
    pub fn new_x509(identifier: &str) -> Result<Self> {
        let mut params = CertificateParams::default(); // PKCS_ECDSA_P256_SHA256
        params.distinguished_name = DistinguishedName::new();
        params
            .distinguished_name
            .push(DnType::CommonName, identifier);

        let certificate =
            Certificate::from_params(params).context("Failed to generate certificate")?;

        let certificate_der = certificate.serialize_der()?;

        let private_key_der = certificate.serialize_private_key_der();

        Ok(Self {
            certificate_der,
            private_key_der,
        })
    }

    pub fn new(private_key_der: Vec<u8>, certificate_der: Vec<u8>) -> Self {
        Self {
            private_key_der,
            certificate_der,
        }
    }

    pub fn to_pem(&self) -> String {
        pem::encode_many(&[
            pem::Pem::new("PRIVATE KEY", self.private_key_der.clone()),
            pem::Pem::new("CERTIFICATE", self.certificate_der.clone()),
        ])
    }

    pub fn to_pfx_der(&self, password: &str, friendly_name: &str) -> Result<Vec<u8>> {
        let pfx = p12::PFX::new(
            &self.certificate_der,
            &self.private_key_der,
            None,
            password,
            friendly_name,
        )
        .context("Failed to convert certificate to PKCS#12 pfx")?;

        Ok(pfx.to_der())
    }
}

type PBKDF2Key = [u8; digest::SHA512_256_OUTPUT_LEN];

// Derive secure key from password & salt
pub struct PBKDF2 {
    key: PBKDF2Key,
}

impl PBKDF2 {
    pub fn derive(password: &str, salt: &str) -> Result<Self> {
        ensure!(
            password.len() >= 8,
            "password must contain at least 8 characters"
        );
        ensure!(salt.len() >= 8, "salt must contain at least 8 characters");

        let key = Self::generate(password.as_bytes(), salt.as_bytes())?;
        Ok(Self { key })
    }

    fn generate(password: &[u8], salt: &[u8]) -> Result<PBKDF2Key> {
        let mut key = [0u8; digest::SHA512_256_OUTPUT_LEN];

        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(100_000).expect("iterations count must be non-zero"),
            salt,
            password,
            &mut key,
        );

        Ok(key)
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
    pub fn new_from_password(password: &str, salt: &str) -> Result<Self> {
        let key = PBKDF2::derive(password, salt)?;

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
}
