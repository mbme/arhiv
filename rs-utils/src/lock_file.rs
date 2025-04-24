use std::{fs, time::Instant};

use anyhow::{bail, Result};

use crate::log;

pub struct LockFile {
    lock: fslock::LockFile,
    file_path: String,
    cleanup: bool,
}

impl LockFile {
    pub fn must_lock(file_path: &str) -> Result<Self> {
        log::debug!("Locking file {file_path}");

        let mut lock = fslock::LockFile::open(file_path)?;

        let locked = lock.try_lock()?;

        if !locked {
            bail!("failed to lock file {file_path}");
        }

        Ok(Self {
            lock,
            file_path: file_path.to_string(),
            cleanup: true,
        })
    }

    pub fn wait_for_lock(file_path: &str) -> Result<Self> {
        log::debug!("Waiting to lock file {file_path}");

        let start_time = Instant::now();

        let mut lock = fslock::LockFile::open(file_path)?;

        lock.lock()?;

        let duration = start_time.elapsed();
        log::trace!("Locked file {file_path} in {:?}", duration);

        Ok(Self {
            lock,
            file_path: file_path.to_string(),
            cleanup: false,
        })
    }
}

impl Drop for LockFile {
    fn drop(&mut self) {
        self.lock.unlock().expect("must unlock");

        if self.cleanup {
            if let Err(err) = fs::remove_file(&self.file_path) {
                log::warn!("Failed to remove Lock file {}: {}", self.file_path, err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::TempFile;

    use super::*;

    #[test]
    fn test_must_lock() {
        let temp_file = TempFile::new();

        let lock = LockFile::must_lock(&temp_file.path).unwrap();
        assert!(
            temp_file.exists(),
            "Lock file should exist after acquiring the first lock"
        );

        temp_file.write_str("test").unwrap();

        let file_content = temp_file.str_contents().unwrap();
        assert_eq!(file_content, "test");

        assert!(LockFile::must_lock(&temp_file.path).is_err());

        drop(lock);
        assert!(
            !temp_file.exists(),
            "Lock file should not exist after releasing the second lock"
        );

        assert!(LockFile::must_lock(&temp_file.path).is_ok());
    }

    #[test]
    fn test_wait_for_lock() {
        let temp_file = TempFile::new();

        // Acquire the first lock
        let lock1 = LockFile::wait_for_lock(&temp_file.path).unwrap();
        assert!(temp_file.exists());

        // Spawn a new thread to attempt to acquire the lock
        let temp_file_clone = temp_file.path.clone();
        let handle = std::thread::spawn(move || LockFile::wait_for_lock(&temp_file_clone).unwrap());

        // Ensure the lock is still held by the first lock
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(!handle.is_finished());

        // Drop the first lock
        drop(lock1);

        // Wait for the second lock to be acquired
        let lock2 = handle.join().unwrap();
        assert!(temp_file.exists());

        // Clean up
        drop(lock2);
        assert!(temp_file.exists());
    }
}
