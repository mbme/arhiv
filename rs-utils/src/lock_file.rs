use std::fs;

use anyhow::{bail, Result};

pub struct LockFile {
    lock: fslock::LockFile,
    file_path: String,
}

impl LockFile {
    pub fn new(file_path: &str) -> Result<Self> {
        log::debug!("Locking file {file_path}");

        let mut lock = fslock::LockFile::open(file_path)?;

        let locked = lock.try_lock()?;

        if !locked {
            bail!("failed to lock file {file_path}");
        }

        Ok(Self {
            lock,
            file_path: file_path.to_string(),
        })
    }
}

impl Drop for LockFile {
    fn drop(&mut self) {
        self.lock.unlock().expect("must unlock");

        if let Err(err) = fs::remove_file(&self.file_path) {
            log::warn!("Failed to remove Lock file {}: {}", self.file_path, err);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{build_path, file_exists, get_temp_dir};

    use super::*;

    #[test]
    fn test_lock_file() {
        let lock_file = build_path(get_temp_dir(), "test-lock-file.lock");

        let lock = LockFile::new(&lock_file).unwrap();
        assert!(file_exists(&lock_file).unwrap());

        fs::write(&lock_file, "test").unwrap();

        let file_content = fs::read_to_string(&lock_file).unwrap();
        assert_eq!(file_content, "test");

        assert!(LockFile::new(&lock_file).is_err());

        drop(lock);
        assert!(!file_exists(&lock_file).unwrap());

        assert!(LockFile::new(&lock_file).is_ok());
    }
}
