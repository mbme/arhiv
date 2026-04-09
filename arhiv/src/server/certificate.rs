use std::{fs, io::Write};

use anyhow::{Context, Result, anyhow};
use baza_common::{
    CryptoKey, ExposeSecret, SecretBytes, SecretString, Timestamp, file_exists, log,
    must_create_file,
};
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair};

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

pub fn read_or_generate_certificate(root_dir: &str) -> Result<SelfSignedCertificate> {
    let cert_path = format!("{root_dir}/arhiv-server.pem");

    if file_exists(&cert_path)? {
        log::info!("Reading Arhiv certificate from {cert_path}");

        let data = fs::read_to_string(cert_path).context("Failed to read certificate file")?;
        let data: SecretString = data.into();

        let certificate = SelfSignedCertificate::from_pem(&data)?;

        Ok(certificate)
    } else {
        log::info!("Generating Arhiv certificate into {cert_path}");

        let certificate = generate_self_signed_certificate()?;

        let data = certificate.to_pem();

        let mut file = must_create_file(&cert_path)
            .context(anyhow!("Failed to create certificate file {cert_path}"))?;
        file.write_all(data.expose_secret().as_bytes())?;
        file.sync_all()?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut perms = file.metadata()?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(cert_path, perms)?;
        }

        Ok(certificate)
    }
}

pub fn generate_ui_crypto_key(certificate_private_key: SecretBytes) -> CryptoKey {
    CryptoKey::derive_subkey(
        certificate_private_key.expose_secret(),
        CryptoKey::salt_from_data("arhiv-server auth token").expect("Must generate salt from data"),
    )
    .expect("Failed to generate ui crypto key")
}

fn generate_self_signed_certificate() -> Result<SelfSignedCertificate> {
    let timestamp = Timestamp::now();
    let certificate_id = format!("Arhiv {timestamp}");

    SelfSignedCertificate::new_x509(&certificate_id)
}
