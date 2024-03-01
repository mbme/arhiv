use core::fmt::Write;
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

use anyhow::{Context, Result};
use base64::{engine, Engine};
use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType};

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
    engine::general_purpose::URL_SAFE.encode(bytes)
}

#[must_use]
pub fn is_valid_base64(value: &str) -> bool {
    engine::general_purpose::URL_SAFE.decode(value).is_ok()
}

#[must_use]
pub fn bytes_to_hex_string(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(2 * bytes.len());
    for byte in bytes {
        write!(result, "{byte:02X}").expect("failed to write data into String");
    }

    result
}

pub struct SelfSignedCertificate {
    pub private_key_der: Vec<u8>,
    pub certificate_der: Vec<u8>,
}

impl SelfSignedCertificate {
    pub fn new_x509(identifier: &str) -> Result<Self> {
        let mut params = CertificateParams::default();
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
}

#[cfg(test)]
mod tests {
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
}
