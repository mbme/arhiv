use core::fmt;
use std::env;
use std::fs;

use anyhow::{Context, Result};
use tokio::fs as tokio_fs;

use crate::build_path;
use crate::get_file_size;
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
        let file_name = generate_temp_file_name(prefix.as_ref(), suffix.as_ref());

        TempFile {
            path: build_path(get_temp_dir(), file_name),
        }
    }

    pub fn new_in_dir(dir: impl AsRef<str>, prefix: impl AsRef<str>) -> Self {
        let file_name = generate_temp_file_name(prefix.as_ref(), "");

        TempFile {
            path: build_path(dir, file_name),
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

    pub async fn open_tokio_file(&self, start_pos: u64) -> Result<tokio_fs::File> {
        let mut options = tokio_fs::OpenOptions::new();
        options.write(true).create(true);

        if start_pos == 0 {
            options.truncate(true);
        } else {
            options.append(true);
        }

        options
            .open(&self.path)
            .await
            .context("failed to open download file for write")
    }

    #[must_use]
    pub fn exists(&self) -> bool {
        path_exists(&self.path)
    }

    pub fn str_contents(&self) -> Result<String> {
        fs::read_to_string(&self.path).context("failed to read file contents")
    }

    pub fn size(&self) -> Result<u64> {
        get_file_size(&self.path)
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

#[must_use]
pub fn get_temp_dir() -> String {
    let path = env::temp_dir();

    path_to_string(path)
}

#[must_use]
pub fn generate_temp_file_name(prefix: &str, suffix: &str) -> String {
    let name = generate_alpanumeric_string(8);

    format!("{prefix}{name}{suffix}")
}

pub fn generate_temp_path(prefix: &str, suffix: &str) -> String {
    let file_name = generate_temp_file_name(prefix, suffix);

    build_path(get_temp_dir(), file_name)
}
