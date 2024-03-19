use anyhow::Result;
use rs_utils::SelfSignedCertificate;

use crate::{settings::SETTINGS_NAMESPACE, BazaConnection, KvsConstKey};

const SETTING_LOGIN: &KvsConstKey<String> = &KvsConstKey::new(SETTINGS_NAMESPACE, "login");
const SETTING_PASSWORD: &KvsConstKey<String> = &KvsConstKey::new(SETTINGS_NAMESPACE, "password");
const SETTING_CERTIFICATE: &KvsConstKey<SelfSignedCertificate> =
    &KvsConstKey::new(SETTINGS_NAMESPACE, "certificate");

impl BazaConnection {
    pub(crate) fn set_login(&self, login: &String) -> Result<()> {
        self.kvs_const_set(SETTING_LOGIN, login)
    }

    pub fn get_login(&self) -> Result<String> {
        self.kvs_const_must_get(SETTING_LOGIN)
    }

    pub(crate) fn set_password(&self, password: &String) -> Result<()> {
        // FIXME secstr password in baza
        self.kvs_const_set(SETTING_PASSWORD, password)
    }

    pub(crate) fn get_password(&self) -> Result<String> {
        // FIXME secstr password in baza
        self.kvs_const_must_get(SETTING_PASSWORD)
    }

    pub(crate) fn set_certificate(&self, certificate: &SelfSignedCertificate) -> Result<()> {
        // FIXME secstr certificate in baza
        self.kvs_const_set(SETTING_CERTIFICATE, certificate)
    }

    pub fn get_certificate(&self) -> Result<SelfSignedCertificate> {
        self.kvs_const_must_get(SETTING_CERTIFICATE)
    }
}
