use anyhow::Result;
use which::which;

use crate::path_to_string;

pub struct EnvCapabilities {
    pub nodejs: bool,
    pub chrome: Option<String>,
}

impl EnvCapabilities {
    pub fn check() -> Result<Self> {
        let nodejs = which("node").is_ok();
        let chrome = which("chromium").ok().map(path_to_string).transpose()?;

        Ok(Self { nodejs, chrome })
    }

    #[must_use]
    pub fn must_check() -> Self {
        EnvCapabilities::check().expect("failed to check capabilities")
    }
}
