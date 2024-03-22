use anyhow::Result;
use rs_utils::SecretString;

use crate::{settings::SETTINGS_NAMESPACE, BazaConnection, KvsConstKey};

const SETTING_LOGIN: &KvsConstKey<String> = &KvsConstKey::new(SETTINGS_NAMESPACE, "login");
const SETTING_PASSWORD: &KvsConstKey<String> = &KvsConstKey::new(SETTINGS_NAMESPACE, "password");

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
}
