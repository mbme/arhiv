use std::{
    fs::{self, File, OpenOptions, TryLockError},
    time::Instant,
};

use anyhow::Result;

use crate::log;

/// Owns an exclusive advisory lock for a lock-file path.
///
/// The lock is held by the open file handle. `must_lock` treats the path as a
/// temporary single-owner marker and removes it on drop; `wait_for_lock` leaves
/// the path in place for callers that use it as durable coordination state.
pub struct LockFile {
    file: Option<File>,
    file_path: String,
    drop_behavior: LockFileDropBehavior,
}

enum LockFileDropBehavior {
    RemoveFile,
    KeepFile,
}

impl LockFile {
    /// Acquires the lock immediately or returns an error if another handle owns it.
    ///
    /// The lock file is removed when this guard is dropped.
    pub fn must_lock(file_path: &str) -> Result<Self> {
        log::debug!("Locking file {file_path}");

        let file = open_lock_file(file_path)?;
        file.try_lock().map_err(|err| match err {
            TryLockError::WouldBlock => anyhow::anyhow!("failed to lock file {file_path}"),
            TryLockError::Error(err) => err.into(),
        })?;

        Ok(Self {
            file: Some(file),
            file_path: file_path.to_string(),
            drop_behavior: LockFileDropBehavior::RemoveFile,
        })
    }

    /// Blocks until the lock can be acquired.
    ///
    /// The lock file remains after this guard is dropped.
    pub fn wait_for_lock(file_path: &str) -> Result<Self> {
        log::debug!("Waiting to lock file {file_path}");

        let start_time = Instant::now();

        let file = open_lock_file(file_path)?;
        file.lock()?;

        let duration = start_time.elapsed();
        log::trace!("Locked file {file_path} in {:?}", duration);

        Ok(Self {
            file: Some(file),
            file_path: file_path.to_string(),
            drop_behavior: LockFileDropBehavior::KeepFile,
        })
    }
}

/// Opens the lock file with read/write permissions required by platform locks.
fn open_lock_file(file_path: &str) -> Result<File> {
    Ok(OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(file_path)?)
}

impl Drop for LockFile {
    fn drop(&mut self) {
        if let Some(file) = self.file.take()
            && let Err(err) = file.unlock()
        {
            log::warn!("Failed to unlock file {}: {}", self.file_path, err);
        }

        if matches!(self.drop_behavior, LockFileDropBehavior::RemoveFile)
            && let Err(err) = fs::remove_file(&self.file_path)
        {
            log::warn!("Failed to remove Lock file {}: {}", self.file_path, err);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::TempFile;

    use super::*;

    #[test]
    fn must_lock_fails_while_another_handle_owns_lock() {
        let temp_file = TempFile::new();

        let lock = LockFile::must_lock(&temp_file.path).unwrap();

        assert!(LockFile::must_lock(&temp_file.path).is_err());

        drop(lock);

        assert!(LockFile::must_lock(&temp_file.path).is_ok());
    }

    #[test]
    fn wait_for_lock_blocks_until_owner_drops_lock() {
        let temp_file = TempFile::new();

        let lock1 = LockFile::wait_for_lock(&temp_file.path).unwrap();

        let temp_file_clone = temp_file.path.clone();
        let handle = std::thread::spawn(move || LockFile::wait_for_lock(&temp_file_clone).unwrap());

        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(!handle.is_finished());

        drop(lock1);

        let lock2 = handle.join().unwrap();
        assert!(temp_file.exists());

        drop(lock2);
    }

    #[test]
    fn must_lock_preserves_contents_while_held_and_removes_file_on_drop() {
        let temp_file = TempFile::new();

        let lock = LockFile::must_lock(&temp_file.path).unwrap();
        temp_file.write_str("test").unwrap();

        assert_eq!(temp_file.str_contents().unwrap(), "test");

        drop(lock);

        assert!(!temp_file.exists());
    }

    #[test]
    fn wait_for_lock_preserves_contents_and_file_on_drop() {
        let temp_file = TempFile::new();

        let lock = LockFile::wait_for_lock(&temp_file.path).unwrap();
        temp_file.write_str("test").unwrap();

        drop(lock);

        assert!(temp_file.exists());
        assert_eq!(temp_file.str_contents().unwrap(), "test");
    }
}
