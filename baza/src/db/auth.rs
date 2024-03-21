use anyhow::Result;
use rs_utils::{SecretString, SelfSignedCertificate};

use crate::{settings::SETTINGS_NAMESPACE, BazaConnection, KvsConstKey};

const SETTING_LOGIN: &KvsConstKey<String> = &KvsConstKey::new(SETTINGS_NAMESPACE, "login");
const SETTING_PASSWORD: &KvsConstKey<String> = &KvsConstKey::new(SETTINGS_NAMESPACE, "password");
const SETTING_CERTIFICATE: &KvsConstKey<Vec<u8>> =
    &KvsConstKey::new(SETTINGS_NAMESPACE, "certificate");
const SETTING_PRIVATE_KEY: &KvsConstKey<Vec<u8>> =
    &KvsConstKey::new(SETTINGS_NAMESPACE, "private_key");

impl BazaConnection {
    pub(crate) fn set_login(&self, login: &String) -> Result<()> {
        self.kvs_const_set(SETTING_LOGIN, login)
    }

    pub fn get_login(&self) -> Result<String> {
        self.kvs_const_must_get(SETTING_LOGIN)
    }

    pub(crate) fn set_password(&self, password: SecretString) -> Result<()> {
        self.kvs_const_set(SETTING_PASSWORD, &password.into_unsecure_string())
    }

    pub(crate) fn get_password(&self) -> Result<SecretString> {
        let password = self.kvs_const_must_get(SETTING_PASSWORD)?;

        Ok(password.into())
    }

    pub(crate) fn set_certificate(&self, certificate: &SelfSignedCertificate) -> Result<()> {
        self.kvs_const_set(SETTING_CERTIFICATE, &certificate.certificate_der)?;
        self.kvs_const_set(
            SETTING_PRIVATE_KEY,
            &certificate.private_key_der.as_bytes().into(),
        )?;

        Ok(())
    }

    pub fn get_certificate(&self) -> Result<SelfSignedCertificate> {
        Ok(SelfSignedCertificate {
            certificate_der: self.kvs_const_must_get(SETTING_CERTIFICATE)?,
            private_key_der: self.kvs_const_must_get(SETTING_PRIVATE_KEY)?.into(),
        })
    }
}
