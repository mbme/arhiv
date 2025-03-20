use std::{fs, io::Write};

use anyhow::{anyhow, Context, Result};

use rs_utils::{
    crypto_key::CryptoKey, file_exists, log, must_create_file, now, ExposeSecret, SecretBytes,
    SecretString, SelfSignedCertificate,
};

pub fn read_or_generate_certificate(root_dir: &str) -> Result<SelfSignedCertificate> {
    let cert_path = format!("{root_dir}/arhiv-server.pem");

    if file_exists(&cert_path)? {
        log::info!("Reading Arhiv certificate from {cert_path}");

        let data = fs::read_to_string(&cert_path).context("Failed to read certificate file")?;
        let data: SecretString = data.into();

        let certificate = SelfSignedCertificate::from_pem(&data)?;

        Ok(certificate)
    } else {
        log::info!("Generating Arhiv certificate into {cert_path}");

        let certificate = generate_certificate()?;

        let data = certificate.to_pem();

        let mut file = must_create_file(&cert_path)
            .context(anyhow!("Failed to create certificate file {cert_path}"))?;
        file.write_all(data.expose_secret().as_bytes())?;
        file.sync_all()?;

        if cfg!(unix) {
            use std::os::unix::fs::PermissionsExt;

            // Set permissions to 600 (only owner can read/write)
            let mut perms = file.metadata()?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&cert_path, perms)?;
        }

        Ok(certificate)
    }
}

fn generate_certificate() -> Result<SelfSignedCertificate> {
    let timestamp = now();
    let certificate_id = format!("Arhiv {timestamp}");

    SelfSignedCertificate::new_x509(&certificate_id)
}

pub fn generate_ui_crypto_key(certificate_private_key: SecretBytes) -> CryptoKey {
    CryptoKey::derive_subkey(
        certificate_private_key.expose_secret(),
        CryptoKey::salt_from_data("arhiv-server auth token").expect("Must generate salt from data"),
    )
    .expect("Failed to generate ui crypto key")
}
