use std::{
    borrow::Borrow,
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
use secstr::{SecUtf8, SecVec};
use serde::Deserialize;

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

#[derive(Clone)]
pub struct SecretBytes(SecVec<u8>);

impl SecretBytes {
    pub fn new(value: Vec<u8>) -> Self {
        Self(SecVec::new(value))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.borrow()
    }

    pub fn len(&self) -> usize {
        self.as_bytes().len()
    }

    pub fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }
}

impl AsRef<[u8]> for SecretBytes {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<Vec<u8>> for SecretBytes {
    fn from(value: Vec<u8>) -> Self {
        Self::new(value)
    }
}

#[derive(Deserialize)]
pub struct SecretString(SecUtf8);

impl SecretString {
    pub fn new(value: impl Into<String>) -> Self {
        Self(SecUtf8::from(value))
    }

    pub fn as_str(&self) -> &str {
        self.0.unsecure()
    }

    pub fn into_unsecure_string(self) -> String {
        self.0.into_unsecure()
    }

    pub fn len(&self) -> usize {
        self.as_str().len()
    }

    pub fn is_empty(&self) -> bool {
        self.as_str().is_empty()
    }
}

impl From<String> for SecretString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl AsRef<[u8]> for SecretString {
    fn as_ref(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

#[derive(Clone)]
pub struct SelfSignedCertificate {
    pub private_key_der: SecretBytes,
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
        let private_key_der = SecretBytes::new(private_key_der);

        Ok(Self {
            certificate_der,
            private_key_der,
        })
    }

    pub fn new(private_key_der: SecretBytes, certificate_der: Vec<u8>) -> Self {
        Self {
            private_key_der,
            certificate_der,
        }
    }

    pub fn to_pem(&self) -> SecretString {
        pem::encode_many(&[
            pem::Pem::new("PRIVATE KEY", self.private_key_der.as_bytes().to_vec()),
            pem::Pem::new("CERTIFICATE", self.certificate_der.clone()),
        ])
        .into()
    }

    pub fn to_pfx_der(&self, password: &SecretString, friendly_name: &str) -> Result<SecretBytes> {
        let pfx = p12::PFX::new(
            &self.certificate_der,
            self.private_key_der.as_bytes(),
            None,
            password.as_str(),
            friendly_name,
        )
        .context("Failed to convert certificate to PKCS#12 pfx")?;

        Ok(pfx.to_der().into())
    }

    pub fn from_pfx_der(password: &SecretString, bytes: SecretBytes) -> Result<Self> {
        let pfx = p12::PFX::parse(bytes.as_ref()).context("Failed to parse PFX")?;

        let mut cert_bags = pfx
            .cert_x509_bags(password.as_str())
            .context("Failed to decrypt the PFX with provided password")?;
        ensure!(
            cert_bags.len() == 1,
            "Expected exactly 1 x509 certificate, got {}",
            cert_bags.len()
        );
        let certificate_der = cert_bags.remove(0);

        let mut key_bags = pfx
            .key_bags(password.as_str())
            .context("Failed to decrypt the PFX with provided password")?;
        ensure!(
            key_bags.len() == 1,
            "Expected exactly 1 private key, got {}",
            key_bags.len()
        );

        let private_key_der = SecretBytes::new(key_bags.remove(0));

        Ok(Self {
            private_key_der,
            certificate_der,
        })
    }
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
}
