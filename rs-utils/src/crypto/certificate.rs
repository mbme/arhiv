use anyhow::{Context, Result};
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair};
use secrecy::ExposeSecret;

use super::{SecretBytes, SecretString};

#[derive(Clone)]
pub struct SelfSignedCertificate {
    pub private_key_der: SecretBytes,
    pub certificate_der: Vec<u8>,
}

impl SelfSignedCertificate {
    pub fn new_x509(identifier: &str) -> Result<Self> {
        let mut params = CertificateParams::default();
        params.distinguished_name = DistinguishedName::new();
        params
            .distinguished_name
            .push(DnType::CommonName, identifier);

        let key_pair = KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256)
            .context("failed to generate a key pair")?;

        let certificate = params
            .self_signed(&key_pair)
            .context("Failed to generate certificate")?;

        let certificate_der = certificate.der().to_vec();

        let private_key_der = key_pair.serialize_der();
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
            pem::Pem::new("PRIVATE KEY", self.private_key_der.expose_secret()),
            pem::Pem::new("CERTIFICATE", self.certificate_der.clone()),
        ])
        .into()
    }

    pub fn from_pem(data: &SecretString) -> Result<Self> {
        let items =
            pem::parse_many(data.expose_secret().as_bytes()).context("Failed to parse .pem")?;

        let private_key = items
            .iter()
            .find(|item| item.tag() == "PRIVATE KEY")
            .context("PRIVATE KEY must be present in .pem")?;
        let private_key_der = SecretBytes::new(private_key.contents().to_vec());

        let certificate = items
            .iter()
            .find(|item| item.tag() == "CERTIFICATE")
            .context("CERTIFICATE must be present in .pem")?;
        let certificate_der = certificate.contents().to_vec();

        Ok(Self {
            private_key_der,
            certificate_der,
        })
    }
}
