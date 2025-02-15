use anyhow::Result;
use rs_utils::SecretString;

use crate::{settings::SETTINGS_NAMESPACE, BazaConnection, KvsConstKey};

const SETTING_PASSWORD: &KvsConstKey<String> = &KvsConstKey::new(SETTINGS_NAMESPACE, "password");

impl BazaConnection {
    pub(crate) fn set_password(&self, password: SecretString) -> Result<()> {
        self.kvs_const_set(SETTING_PASSWORD, password.as_string_ref())
    }

    pub(crate) fn get_password(&self) -> Result<SecretString> {
        let password = self.kvs_const_must_get(SETTING_PASSWORD)?;

        Ok(password.into())
    }
}
