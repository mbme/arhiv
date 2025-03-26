use anyhow::Result;

use rs_utils::{
    crypto_key::CryptoKey, ExposeSecret, SecretBytes, SelfSignedCertificate, Timestamp,
};

pub fn generate_ui_crypto_key(certificate_private_key: SecretBytes) -> CryptoKey {
    CryptoKey::derive_subkey(
        certificate_private_key.expose_secret(),
        CryptoKey::salt_from_data("arhiv-server auth token").expect("Must generate salt from data"),
    )
    .expect("Failed to generate ui crypto key")
}

pub fn generate_certificate() -> Result<SelfSignedCertificate> {
    let timestamp = Timestamp::now();
    let certificate_id = format!("Arhiv {timestamp}");

    SelfSignedCertificate::new_x509(&certificate_id)
}
