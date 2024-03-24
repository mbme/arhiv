use core::fmt;
use std::env;
use std::fs;

use anyhow::{Context, Result};

use crate::path_to_string;
use crate::{generate_alpanumeric_string, path_exists};

pub struct TempFile {
    pub path: String,
}

impl TempFile {
    #[must_use]
    pub fn new() -> Self {
        TempFile::new_with_details("TempFile-", "")
    }

    pub fn new_with_details(prefix: impl AsRef<str>, suffix: impl AsRef<str>) -> Self {
        TempFile {
            path: generate_temp_path(prefix.as_ref(), suffix.as_ref()),
        }
    }

    pub fn mkdir(&self) -> Result<()> {
        fs::create_dir(&self.path)?;

        Ok(())
    }

    pub fn write(&self, data: &[u8]) -> Result<()> {
        fs::write(&self.path, data)?;

        Ok(())
    }

    pub fn write_str(&self, data: impl AsRef<str>) -> Result<()> {
        self.write(data.as_ref().as_bytes())
    }

    pub fn create_file(&self) -> Result<()> {
        fs::File::create(&self.path).context("failed to create file")?;

        Ok(())
    }

    #[must_use]
    pub fn exists(&self) -> bool {
        path_exists(&self.path)
    }

    pub fn str_contents(&self) -> Result<String> {
        fs::read_to_string(&self.path).context("failed to read file contents")
    }
}

impl Default for TempFile {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let metadata = if let Ok(metadata) = fs::symlink_metadata(&self.path) {
            metadata
        } else {
            // file doesn't exist or is inaccessible
            return;
        };

        if metadata.is_dir() {
            fs::remove_dir_all(&self.path).expect("failed to remove dir");
        } else {
            fs::remove_file(&self.path).expect("failed to remove file");
        }
    }
}

impl AsRef<str> for TempFile {
    fn as_ref(&self) -> &str {
        &self.path
    }
}

impl fmt::Display for TempFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.path)
    }
}

pub fn file_in_temp_dir(file_name: impl AsRef<str>) -> String {
    let mut path = env::temp_dir();

    path.push(file_name.as_ref());

    path_to_string(path).expect("must be able to convert path to string")
}

#[must_use]
pub fn generate_temp_path(prefix: &str, suffix: &str) -> String {
    let name = generate_alpanumeric_string(8);

    file_in_temp_dir(format!("{prefix}{name}{suffix}"))
}
