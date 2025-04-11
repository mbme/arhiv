use std::sync::Arc;

use anyhow::Result;

use baza::DEV_MODE;
use rs_utils::{
    keyring::{Keyring, NoopKeyring, SystemKeyring},
    SecretString,
};

pub struct ArhivKeyring {
    keyring: Arc<dyn Keyring + Send + Sync>,
}

impl ArhivKeyring {
    pub const PASSWORD: &str = "password";

    pub fn new(keyring: Arc<dyn Keyring + Send + Sync>) -> ArhivKeyring {
        ArhivKeyring { keyring }
    }

    pub fn new_noop() -> ArhivKeyring {
        ArhivKeyring::new(Arc::new(NoopKeyring))
    }

    pub fn new_system_keyring() -> ArhivKeyring {
        ArhivKeyring::new(Arc::new(SystemKeyring::new(if DEV_MODE {
            "Arhiv-dev"
        } else {
            "Arhiv"
        })))
    }

    pub fn get_password(&self) -> Result<Option<SecretString>> {
        self.keyring.get_string(ArhivKeyring::PASSWORD)
    }

    pub fn set_password(&self, password: Option<SecretString>) -> Result<()> {
        self.keyring.set_string(ArhivKeyring::PASSWORD, password)
    }
}
