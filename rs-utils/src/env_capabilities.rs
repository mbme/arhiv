use anyhow::Result;
use which::which;

use crate::path_to_string;

pub struct EnvCapabilities {
    pub nodejs: bool,
    pub chrome: Option<String>,
    pub zstd: bool,
}

impl EnvCapabilities {
    pub fn check() -> Result<Self> {
        let nodejs = which("node").is_ok();
        let chrome = which("chromium").ok().map(path_to_string).transpose()?;
        let zstd = which("zstd").is_ok();

        Ok(Self {
            nodejs,
            chrome,
            zstd,
        })
    }

    #[must_use]
    pub fn must_check() -> Self {
        EnvCapabilities::check().expect("failed to check capabilities")
    }
}
