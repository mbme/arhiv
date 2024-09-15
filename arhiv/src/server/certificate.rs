use std::{fs, io::Write};

use anyhow::{anyhow, Context, Result};

use rs_utils::{
    file_exists, log, must_create_file, now, CryptoKey, SecretBytes, SecretString,
    SelfSignedCertificate, HMAC,
};

pub fn read_or_generate_certificate(root_dir: &str) -> Result<SelfSignedCertificate> {
    let cert_path = format!("{root_dir}/arhiv-server.pem");

    if file_exists(&cert_path)? {
        let data = fs::read_to_string(&cert_path).context("Failed to read certificate file")?;
        let data = SecretString::new(data);

        let certificate = SelfSignedCertificate::from_pem(&data)?;

        log::info!("Read arhiv certificate from {cert_path}");

        Ok(certificate)
    } else {
        let certificate = generate_certificate()?;

        let data = certificate.to_pem();

        let mut file = must_create_file(&cert_path)
            .context(anyhow!("Failed to create certificate file {cert_path}"))?;
        file.write_all(data.as_ref())?;
        file.sync_all()?;

        log::info!("Wrote arhiv certificate into {cert_path}");

        Ok(certificate)
    }
}

fn generate_certificate() -> Result<SelfSignedCertificate> {
    let timestamp = now();
    let certificate_id = format!("Arhiv {timestamp}");

    SelfSignedCertificate::new_x509(&certificate_id)
}

pub fn generate_ui_key_verifier(certificate_private_key: SecretBytes) -> Result<HMAC> {
    let key = CryptoKey::from_crypto_bytes(certificate_private_key, None)?;

    let subkey = key.derive_subkey("arhiv-server auth token")?;

    HMAC::new(subkey)
}
