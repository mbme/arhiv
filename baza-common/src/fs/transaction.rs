//! Best-effort rollback for grouped filesystem mutations.
//!
//! Transactions compose durable single-file operations and record undo steps.
//! They are an in-process guard, not a crash-safe journal.

use std::{
    fs,
    io::{ErrorKind, Write},
    path::PathBuf,
};

use anyhow::{Context, Result, bail, ensure};

use super::durable::{
    backup_path, copy_to_new_file, move_file_durable, path_display, remove_file_durable,
    remove_file_unsynced, sync_parent_dir,
};
use super::{
    dir_exists, ensure_file_exists, file_exists, get_file_size, must_create_file, path_to_string,
    set_file_size,
};
use crate::log;

enum FsOperation {
    Backup { src: PathBuf, dest: PathBuf },
    Move { src: PathBuf, dest: PathBuf },
    Copy { src: PathBuf, dest: PathBuf },
    HardLink { src: PathBuf, dest: PathBuf },
    CreateFile { path: PathBuf },
    CreateDir { path: PathBuf },
    AppendFile { path: PathBuf, original_size: u64 },
}

enum MoveRollbackKind {
    Backup,
    Move,
}

/// Best-effort in-process rollback guard for local filesystem mutations.
///
/// `FsTransaction` records undo steps after each successful mutation and replays
/// them in reverse order from `rollback()` or `Drop` unless `commit()` is called.
/// Operations are recorded after the namespace mutation succeeds and before
/// durability syncs that can fail, so sync failures still leave rollback state.
/// Callers must hold the relevant application-level lock before mutating shared
/// paths. This guard is not a crash-safe journal, does not coordinate concurrent
/// transactions, and does not provide atomicity across filesystems or arbitrary
/// directory trees.
pub struct FsTransaction {
    ops: Vec<FsOperation>,
}

// Callers provide external synchronization for shared paths. This type records
// undo steps but does not lock paths or coordinate concurrent transactions.
impl FsTransaction {
    /// Starts an empty transaction with no recorded rollback steps.
    ///
    /// Dropping the transaction rolls back later mutations unless `commit()` clears
    /// the undo log first.
    #[must_use]
    pub fn new() -> FsTransaction {
        FsTransaction { ops: vec![] }
    }

    /// Moves `src` to a unique backup path and restores it on rollback.
    ///
    /// This is useful for staging caller-visible backups that should survive until
    /// commit cleanup or be moved back into place if the transaction aborts.
    pub fn move_to_backup(&mut self, src: impl Into<PathBuf>) -> Result<PathBuf> {
        let src = src.into();
        let dest = backup_path(&src);

        ensure!(!dest.exists(), "backup path must not exist");

        self.move_file_recorded(src.clone(), dest.clone(), MoveRollbackKind::Backup)
            .with_context(|| {
                format!(
                    "Failed to Backup {} to {}",
                    path_display(&src),
                    path_display(&dest)
                )
            })?;

        log::debug!(
            "Backed up {} to {}",
            path_display(&src),
            path_display(&dest)
        );
        Ok(dest)
    }

    /// Moves `src` into `dest` and restores the prior state on rollback.
    ///
    /// Existing destinations are backed up unless `fail_if_dest_exists` requests
    /// strict creation semantics. Cross-filesystem moves are copied and removed in
    /// transaction-owned steps so rollback state matches each visible mutation.
    pub fn move_file(
        &mut self,
        src: impl Into<PathBuf>,
        dest: impl Into<PathBuf>,
        fail_if_dest_exists: bool,
    ) -> Result<()> {
        let src = src.into();
        let dest = dest.into();

        if dest.exists() {
            if fail_if_dest_exists {
                bail!("Can't move file to {}: already exists", path_display(&dest));
            }
            self.move_to_backup(&dest)?;
        }

        self.move_file_recorded(src.clone(), dest.clone(), MoveRollbackKind::Move)
            .with_context(|| {
                format!(
                    "Failed to Move {} to {}",
                    path_display(&src),
                    path_display(&dest)
                )
            })?;

        log::debug!("Moved {} to {}", path_display(&src), path_display(&dest));

        Ok(())
    }

    /// Copies `src` into `dest` and removes the copied destination on rollback.
    ///
    /// Existing destinations are first moved to a transaction backup so rollback
    /// can restore their original contents.
    pub fn copy_file(&mut self, src: impl Into<PathBuf>, dest: impl Into<PathBuf>) -> Result<()> {
        let src = src.into();
        let dest = dest.into();

        if dest.exists() {
            self.move_to_backup(&dest)?;
        }

        let mut reader = fs::File::open(&src)
            .with_context(|| format!("Failed to open Copy source {}", path_display(&src)))?;
        copy_to_new_file(&mut reader, &dest).with_context(|| {
            format!(
                "Failed to Copy {} to {}",
                path_display(&src),
                path_display(&dest)
            )
        })?;

        self.ops.push(FsOperation::Copy {
            src: src.clone(),
            dest: dest.clone(),
        });
        sync_parent_dir(&dest)?;
        log::debug!("Copied {} to {}", path_display(&src), path_display(&dest));

        Ok(())
    }

    /// Copies `src` to a newly-created `dest`, failing when `dest` already exists.
    ///
    /// Rollback removes the new destination. Use this when replacing an existing
    /// file would hide a caller or synchronization error.
    pub fn copy_file_new(
        &mut self,
        src: impl Into<PathBuf>,
        dest: impl Into<PathBuf>,
    ) -> Result<()> {
        let src = src.into();
        let dest = dest.into();

        let mut reader = fs::File::open(&src)
            .with_context(|| format!("Failed to open Copy source {}", path_display(&src)))?;
        copy_to_new_file(&mut reader, &dest).with_context(|| {
            format!(
                "Failed to Copy {} to {}",
                path_display(&src),
                path_display(&dest)
            )
        })?;

        self.ops.push(FsOperation::Copy {
            src: src.clone(),
            dest: dest.clone(),
        });
        sync_parent_dir(&dest)?;
        log::debug!("Copied {} to {}", path_display(&src), path_display(&dest));

        Ok(())
    }

    /// Creates a hard link at `dest` and removes that link on rollback.
    ///
    /// Existing destinations are backed up first. The source file is left in place
    /// for both commit and rollback.
    pub fn hard_link_file(
        &mut self,
        src: impl Into<PathBuf>,
        dest: impl Into<PathBuf>,
    ) -> Result<()> {
        let src = src.into();
        let dest = dest.into();

        if dest.exists() {
            self.move_to_backup(&dest)?;
        }

        fs::hard_link(&src, &dest).with_context(|| {
            format!(
                "Failed to HardLink {} to {}",
                path_display(&src),
                path_display(&dest)
            )
        })?;
        self.ops.push(FsOperation::HardLink {
            src: src.clone(),
            dest: dest.clone(),
        });
        sync_parent_dir(&dest)?;
        log::debug!(
            "Hard Linked {} to {}",
            path_display(&src),
            path_display(&dest)
        );

        Ok(())
    }

    /// Removes a file by moving it to a transaction backup.
    ///
    /// Commit deletes the backup; rollback moves it back to the original path.
    pub fn remove_file(&mut self, src: impl Into<PathBuf>) -> Result<()> {
        let src = src.into();

        self.move_to_backup(&src)?;

        log::debug!("Removed file {}", path_display(&src));

        Ok(())
    }

    /// Removes an existing file and records enough state to restore it on rollback.
    ///
    /// Missing files are treated as an intentional no-op and do not add rollback
    /// work.
    pub fn remove_file_if_exists(&mut self, src: impl Into<PathBuf>) -> Result<()> {
        let src = src.into();

        if file_exists(&path_to_string(&src))? {
            self.remove_file(src)?;
        }

        Ok(())
    }

    /// Creates a new file with `data` and removes it on rollback.
    ///
    /// Rollback is recorded after file creation and before fallible writes or
    /// durability syncs, so later rollback removes partial created files.
    pub fn create_file(&mut self, path: impl Into<PathBuf>, data: &[u8]) -> Result<()> {
        let path = path.into();

        let mut file = must_create_file(&path_to_string(&path))
            .with_context(|| format!("Failed to Create file {}", path_display(&path)))?;
        self.ops
            .push(FsOperation::CreateFile { path: path.clone() });

        if !data.is_empty() {
            file.write_all(data).with_context(|| {
                format!("Failed to write data into file {}", path_display(&path))
            })?;
        }

        file.sync_all()
            .context("Failed to sync file changes to disk")?;
        sync_parent_dir(&path)?;

        log::debug!("Created file {}", path_display(&path));

        Ok(())
    }

    /// Creates a single directory and removes it on rollback.
    ///
    /// The directory must be empty by the time rollback runs; this transaction
    /// records directory creation, not ownership of arbitrary directory trees.
    pub fn create_dir(&mut self, path: impl Into<PathBuf>) -> Result<()> {
        let path = path.into();

        fs::create_dir(&path)
            .with_context(|| format!("Failed to Create dir {}", path_display(&path)))?;
        self.ops.push(FsOperation::CreateDir { path: path.clone() });
        sync_parent_dir(&path)?;
        log::debug!("Created dir {}", path_display(&path));

        Ok(())
    }

    /// Creates a missing directory and removes it again on rollback.
    ///
    /// Existing directories are treated as an intentional no-op so rollback only
    /// removes directories created by this transaction.
    pub fn create_dir_if_missing(&mut self, path: impl Into<PathBuf>) -> Result<()> {
        let path = path.into();

        if !dir_exists(&path_to_string(&path))? {
            self.create_dir(path)?;
        }

        Ok(())
    }

    /// Appends bytes to an existing file and truncates back to the original size on rollback.
    ///
    /// This rollback model assumes the transaction owns the file for the duration
    /// of the append; concurrent appends after this call would also be truncated.
    pub fn append_file(&mut self, path: impl Into<PathBuf>, data: &[u8]) -> Result<()> {
        let path = path.into();

        ensure_file_exists(&path_to_string(&path))?;

        let original_size = get_file_size(&path_to_string(&path))?;
        self.ops.push(FsOperation::AppendFile {
            path: path.clone(),
            original_size,
        });

        let mut file = fs::OpenOptions::new()
            .read(false)
            .append(true)
            .create_new(false)
            .open(&path)
            .with_context(|| format!("Failed to open file {}", path_display(&path)))?;

        file.write_all(data)
            .with_context(|| format!("Failed to append data to file {}", path_display(&path)))?;

        file.sync_all()
            .context("Failed to sync file changes to disk")?;

        log::debug!(
            "Appended {} bytes to file {}",
            data.len(),
            path_display(&path)
        );

        Ok(())
    }

    /// Replays recorded undo steps in reverse order and clears the transaction.
    ///
    /// Rollback is best-effort across all recorded operations: it attempts every
    /// undo step, logs individual failures, and returns an error if any step could
    /// not be reverted.
    pub fn rollback(&mut self) -> Result<()> {
        if self.ops.is_empty() {
            return Ok(());
        }

        log::warn!("Reverting {} operations", &self.ops.len());

        let mut failed_count = 0;
        let total_count = self.ops.len();

        // rollback all operations in reverse order
        for op in self.ops.iter().rev() {
            match op {
                FsOperation::Move { src, dest } => {
                    if let Err(err) = move_file_durable(dest, src) {
                        log::error!(
                            "Failed to revert Move {} to {}: {}",
                            path_display(src),
                            path_display(dest),
                            err
                        );
                        failed_count += 1;
                    } else {
                        log::warn!(
                            "Reverted Move {} to {}",
                            path_display(src),
                            path_display(dest)
                        );
                    }
                }

                FsOperation::Copy { src, dest } => {
                    if let Err(err) = remove_file_durable(dest) {
                        log::error!(
                            "Failed to revert Copy {} to {}: {}",
                            path_display(src),
                            path_display(dest),
                            err
                        );
                        failed_count += 1;
                    } else {
                        log::warn!(
                            "Reverted Copy {} to {}",
                            path_display(src),
                            path_display(dest)
                        );
                    }
                }

                FsOperation::HardLink { src, dest } => {
                    if let Err(err) = remove_file_durable(dest) {
                        log::error!(
                            "Failed to revert HardLink {} to {}: {}",
                            path_display(src),
                            path_display(dest),
                            err
                        );
                        failed_count += 1;
                    } else {
                        log::warn!(
                            "Reverted HardLink {} to {}",
                            path_display(src),
                            path_display(dest)
                        );
                    }
                }

                FsOperation::Backup { src, dest } => {
                    if let Err(err) = move_file_durable(dest, src) {
                        log::error!(
                            "Failed to revert Backup {} to {}: {}",
                            path_display(src),
                            path_display(dest),
                            err
                        );
                        failed_count += 1;
                    } else {
                        log::warn!(
                            "Reverted Backup {} to {}",
                            path_display(src),
                            path_display(dest)
                        );
                    }
                }
                FsOperation::CreateFile { path } => {
                    if let Err(err) = remove_file_durable(path) {
                        log::error!("Failed to revert CreateFile {}: {err}", path_display(path));
                        failed_count += 1;
                    } else {
                        log::warn!("Reverted CreateFile {}", path_display(path));
                    }
                }
                FsOperation::CreateDir { path } => {
                    if let Err(err) = fs::remove_dir(path).and_then(|_| sync_parent_dir(path)) {
                        log::error!("Failed to revert CreateDir {}: {err}", path_display(path));
                        failed_count += 1;
                    } else {
                        log::warn!("Reverted CreateDir {}", path_display(path));
                    }
                }
                FsOperation::AppendFile {
                    path,
                    original_size,
                } => {
                    if let Err(err) = set_file_size(&path_to_string(path), *original_size) {
                        log::error!("Failed to revert AppendFile {}: {err}", path_display(path));
                        failed_count += 1;
                    } else {
                        log::warn!("Reverted AppendFile {}", path_display(path));
                    }
                }
            }
        }

        self.ops.clear();

        ensure!(
            failed_count == 0,
            "Failed to revert {} operation(s) out of {}",
            failed_count,
            total_count
        );

        Ok(())
    }

    fn move_file_recorded(
        &mut self,
        src: PathBuf,
        dest: PathBuf,
        rollback_kind: MoveRollbackKind,
    ) -> Result<()> {
        match fs::rename(&src, &dest) {
            Ok(()) => {
                self.push_move_rollback(rollback_kind, src.clone(), dest.clone());
                sync_parent_dir(&src)?;
                sync_parent_dir(&dest)?;
            }
            Err(err) if err.kind() == ErrorKind::CrossesDevices => {
                let mut reader = fs::File::open(&src).with_context(|| {
                    format!("Failed to open Move source {}", path_display(&src))
                })?;
                copy_to_new_file(&mut reader, &dest)?;
                self.ops.push(FsOperation::Copy {
                    src: src.clone(),
                    dest: dest.clone(),
                });
                sync_parent_dir(&dest)?;

                remove_file_unsynced(&src)?;
                self.ops.pop();
                self.push_move_rollback(rollback_kind, src.clone(), dest.clone());
                sync_parent_dir(&src)?;
                sync_parent_dir(&dest)?;
            }
            Err(err) => {
                return Err(err).with_context(|| {
                    format!(
                        "Failed to rename {} to {}",
                        path_display(&src),
                        path_display(&dest)
                    )
                });
            }
        }

        Ok(())
    }

    fn push_move_rollback(&mut self, rollback_kind: MoveRollbackKind, src: PathBuf, dest: PathBuf) {
        match rollback_kind {
            MoveRollbackKind::Backup => self.ops.push(FsOperation::Backup { src, dest }),
            MoveRollbackKind::Move => self.ops.push(FsOperation::Move { src, dest }),
        }
    }

    /// Marks the transaction successful and removes temporary backup files.
    ///
    /// Commit clears rollback state even if backup cleanup fails; cleanup failures
    /// are logged because the caller-visible mutations have already succeeded.
    pub fn commit(&mut self) -> Result<()> {
        for op in &self.ops {
            if let FsOperation::Backup { dest, .. } = op
                && let Err(err) = remove_file_durable(dest)
            {
                log::warn!(
                    "Committed transaction, but failed to remove Backup {}: {}",
                    path_display(dest),
                    err
                );
            }
        }

        self.ops.clear();

        Ok(())
    }
}
#[allow(unused_must_use)]
impl Drop for FsTransaction {
    fn drop(&mut self) {
        self.rollback();
    }
}

impl Default for FsTransaction {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Read};

    use crate::TempFile;

    use super::super::durable::{
        clear_parent_dir_sync_failure, copy_to_new_file, fail_parent_dir_sync_after,
    };
    use super::super::{AtomicFileWriter, replace_file_atomically};
    use super::*;

    #[test]
    fn test_move() -> Result<()> {
        // commit move transaction
        {
            let temp1 = TempFile::new();
            temp1.write_str("temp1")?;

            let temp2 = TempFile::new();

            let mut fs_tx = FsTransaction::new();
            fs_tx.move_file(temp1.as_ref(), temp2.as_ref(), false)?;
            fs_tx.commit()?;

            assert!(!temp1.exists());
            assert!(temp2.exists());
            assert_eq!(temp2.str_contents()?, "temp1");
        }

        // move to existing file
        {
            let temp1 = TempFile::new();
            temp1.write_str("temp1")?;

            let temp2 = TempFile::new();
            temp2.create_file()?;

            let mut fs_tx = FsTransaction::new();
            assert!(
                fs_tx
                    .move_file(temp1.as_ref(), temp2.as_ref(), true)
                    .is_err()
            );

            assert_eq!(temp1.str_contents()?, "temp1");
            assert_eq!(temp2.str_contents()?, "");
        }

        // revert move transaction & restore backup
        {
            let temp1 = TempFile::new();
            temp1.write_str("temp1")?;

            let temp2 = TempFile::new();
            temp2.write_str("temp2")?;

            let mut fs_tx = FsTransaction::new();
            fs_tx.move_file(temp1.as_ref(), temp2.as_ref(), false)?;

            assert!(!temp1.exists());
            assert_eq!(temp2.str_contents()?, "temp1");

            fs_tx.rollback()?;

            assert!(temp1.exists());
            assert_eq!(temp1.str_contents()?, "temp1");
            assert_eq!(temp2.str_contents()?, "temp2");
        }

        Ok(())
    }

    #[test]
    fn test_copy() -> Result<()> {
        // commit copy transaction
        {
            let temp1 = TempFile::new();
            temp1.write_str("temp1")?;

            let temp2 = TempFile::new();

            let mut fs_tx = FsTransaction::new();
            fs_tx.copy_file(temp1.as_ref(), temp2.as_ref())?;
            fs_tx.commit()?;

            assert!(temp1.exists());
            assert!(temp2.exists());
            assert_eq!(temp2.str_contents()?, "temp1");
        }

        // revert copy transaction & restore backup
        {
            let temp1 = TempFile::new();
            temp1.write_str("temp1")?;

            let temp2 = TempFile::new();
            temp2.write_str("temp2")?;

            let mut fs_tx = FsTransaction::new();
            fs_tx.copy_file(temp1.as_ref(), temp2.as_ref())?;

            assert_eq!(temp1.str_contents()?, "temp1");
            assert_eq!(temp2.str_contents()?, "temp1");

            fs_tx.rollback()?;

            assert_eq!(temp1.str_contents()?, "temp1");
            assert_eq!(temp2.str_contents()?, "temp2");
        }

        Ok(())
    }

    #[test]
    fn copy_file_new_preserves_existing_destination() -> Result<()> {
        let temp1 = TempFile::new();
        temp1.write_str("temp1")?;

        let temp2 = TempFile::new();
        temp2.write_str("temp2")?;

        let mut fs_tx = FsTransaction::new();
        assert!(fs_tx.copy_file_new(temp1.as_ref(), temp2.as_ref()).is_err());

        assert_eq!(temp1.str_contents()?, "temp1");
        assert_eq!(temp2.str_contents()?, "temp2");

        Ok(())
    }

    #[test]
    fn copy_file_new_rolls_back_new_destination() -> Result<()> {
        let temp1 = TempFile::new();
        temp1.write_str("temp1")?;

        let temp2 = TempFile::new();

        let mut fs_tx = FsTransaction::new();
        fs_tx.copy_file_new(temp1.as_ref(), temp2.as_ref())?;

        assert_eq!(temp1.str_contents()?, "temp1");
        assert_eq!(temp2.str_contents()?, "temp1");

        fs_tx.rollback()?;

        assert_eq!(temp1.str_contents()?, "temp1");
        assert!(!temp2.exists());

        Ok(())
    }

    #[test]
    fn move_file_rolls_back_after_parent_sync_failure() -> Result<()> {
        let temp1 = TempFile::new();
        temp1.write_str("temp1")?;
        let temp2 = TempFile::new();

        let mut fs_tx = FsTransaction::new();
        fail_parent_dir_sync_after(0);
        let err = fs_tx
            .move_file(temp1.as_ref(), temp2.as_ref(), false)
            .expect_err("parent directory sync failure should fail the move");
        clear_parent_dir_sync_failure();

        assert!(format!("{err:?}").contains("injected parent directory sync failure"));
        assert!(!temp1.exists());
        assert_eq!(temp2.str_contents()?, "temp1");

        fs_tx.rollback()?;

        assert_eq!(temp1.str_contents()?, "temp1");
        assert!(!temp2.exists());

        Ok(())
    }

    #[test]
    fn remove_file_rolls_back_after_backup_parent_sync_failure() -> Result<()> {
        let temp = TempFile::new();
        temp.write_str("temp")?;

        let mut fs_tx = FsTransaction::new();
        fail_parent_dir_sync_after(0);
        let err = fs_tx
            .remove_file(temp.as_ref())
            .expect_err("parent directory sync failure should fail the backup move");
        clear_parent_dir_sync_failure();

        assert!(format!("{err:?}").contains("injected parent directory sync failure"));
        assert!(!temp.exists());

        fs_tx.rollback()?;

        assert_eq!(temp.str_contents()?, "temp");

        Ok(())
    }

    #[test]
    fn copy_file_new_rolls_back_after_parent_sync_failure() -> Result<()> {
        let temp1 = TempFile::new();
        temp1.write_str("temp1")?;
        let temp2 = TempFile::new();

        let mut fs_tx = FsTransaction::new();
        fail_parent_dir_sync_after(0);
        let err = fs_tx
            .copy_file_new(temp1.as_ref(), temp2.as_ref())
            .expect_err("parent directory sync failure should fail the copy");
        clear_parent_dir_sync_failure();

        assert!(format!("{err:?}").contains("injected parent directory sync failure"));
        assert_eq!(temp1.str_contents()?, "temp1");
        assert_eq!(temp2.str_contents()?, "temp1");

        fs_tx.rollback()?;

        assert_eq!(temp1.str_contents()?, "temp1");
        assert!(!temp2.exists());

        Ok(())
    }

    #[test]
    fn copy_file_new_removes_partial_destination_on_copy_error() {
        struct FailingReader {
            emitted_partial: bool,
        }

        impl Read for FailingReader {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                if self.emitted_partial {
                    return Err(io::Error::other("copy failed"));
                }

                self.emitted_partial = true;
                let bytes = b"partial";
                buf[..bytes.len()].copy_from_slice(bytes);
                Ok(bytes.len())
            }
        }

        let temp = TempFile::new();
        let mut reader = FailingReader {
            emitted_partial: false,
        };

        assert!(copy_to_new_file(&mut reader, temp.as_ref()).is_err());
        assert!(!temp.exists());
    }

    #[test]
    fn replace_file_atomically_replaces_existing_file() -> Result<()> {
        let temp = TempFile::new();
        temp.write_str("old")?;

        replace_file_atomically(temp.as_ref(), b"new")?;

        assert_eq!(temp.str_contents()?, "new");

        Ok(())
    }

    #[test]
    fn replace_file_atomically_creates_missing_file() -> Result<()> {
        let temp = TempFile::new();

        replace_file_atomically(temp.as_ref(), b"new")?;

        assert_eq!(temp.str_contents()?, "new");

        Ok(())
    }

    #[test]
    fn atomic_file_writer_preserves_existing_file_on_write_error() -> Result<()> {
        let temp = TempFile::new();
        temp.write_str("old")?;

        let mut writer = AtomicFileWriter::create(temp.as_ref())?;
        writer.write_all(b"partial")?;
        let err = anyhow::anyhow!("write failed");
        drop(writer);

        assert!(format!("{err:?}").contains("write failed"));
        assert_eq!(temp.str_contents()?, "old");

        Ok(())
    }

    #[test]
    fn test_hard_link() -> Result<()> {
        // commit hard link transaction
        {
            let temp1 = TempFile::new();
            temp1.write_str("temp1")?;

            let temp2 = TempFile::new();

            let mut fs_tx = FsTransaction::new();
            fs_tx.hard_link_file(temp1.as_ref(), temp2.as_ref())?;
            fs_tx.commit()?;

            assert!(temp1.exists());
            assert!(temp2.exists());
            assert_eq!(temp2.str_contents()?, "temp1");
        }

        // revert hard link transaction & restore backup
        {
            let temp1 = TempFile::new();
            temp1.write_str("temp1")?;

            let temp2 = TempFile::new();
            temp2.write_str("temp2")?;

            let mut fs_tx = FsTransaction::new();
            fs_tx.hard_link_file(temp1.as_ref(), temp2.as_ref())?;

            assert_eq!(temp1.str_contents()?, "temp1");
            assert_eq!(temp2.str_contents()?, "temp1");

            fs_tx.rollback()?;

            assert_eq!(temp1.str_contents()?, "temp1");
            assert_eq!(temp2.str_contents()?, "temp2");
        }

        Ok(())
    }

    #[test]
    fn hard_link_file_rolls_back_after_parent_sync_failure() -> Result<()> {
        let temp1 = TempFile::new();
        temp1.write_str("temp1")?;
        let temp2 = TempFile::new();

        let mut fs_tx = FsTransaction::new();
        fail_parent_dir_sync_after(0);
        let err = fs_tx
            .hard_link_file(temp1.as_ref(), temp2.as_ref())
            .expect_err("parent directory sync failure should fail the hard link");
        clear_parent_dir_sync_failure();

        assert!(format!("{err:?}").contains("injected parent directory sync failure"));
        assert_eq!(temp1.str_contents()?, "temp1");
        assert_eq!(temp2.str_contents()?, "temp1");

        fs_tx.rollback()?;

        assert_eq!(temp1.str_contents()?, "temp1");
        assert!(!temp2.exists());

        Ok(())
    }

    #[test]
    fn test_remove() -> Result<()> {
        // commit remove transaction
        {
            let temp1 = TempFile::new();
            temp1.write_str("temp1")?;

            let mut fs_tx = FsTransaction::new();
            fs_tx.remove_file(temp1.as_ref())?;
            fs_tx.commit()?;

            assert!(!temp1.exists());
        }

        // revert remove transaction
        {
            let temp1 = TempFile::new();
            temp1.write_str("temp1")?;

            let mut fs_tx = FsTransaction::new();
            fs_tx.remove_file(temp1.as_ref())?;

            assert!(!temp1.exists());

            fs_tx.rollback()?;

            assert_eq!(temp1.str_contents()?, "temp1");
        }

        Ok(())
    }

    #[test]
    fn remove_file_if_exists_is_transactional_noop_for_missing_files() -> Result<()> {
        let missing = TempFile::new();
        let existing = TempFile::new();
        existing.write_str("existing")?;

        let mut fs_tx = FsTransaction::new();
        fs_tx.remove_file_if_exists(missing.as_ref())?;
        fs_tx.remove_file_if_exists(existing.as_ref())?;

        assert!(!missing.exists());
        assert!(!existing.exists());

        fs_tx.rollback()?;

        assert!(!missing.exists());
        assert_eq!(existing.str_contents()?, "existing");

        Ok(())
    }

    #[test]
    fn test_create_file() -> Result<()> {
        // commit create_file transaction
        {
            let temp1 = TempFile::new();

            let mut fs_tx = FsTransaction::new();
            fs_tx.create_file(&temp1.path, "temp1".as_bytes())?;
            fs_tx.commit()?;

            assert!(temp1.exists());
            assert_eq!(temp1.str_contents()?, "temp1");
        }

        // revert create_file transaction
        {
            let temp1 = TempFile::new();

            let mut fs_tx = FsTransaction::new();
            fs_tx.create_file(&temp1.path, "temp1".as_bytes())?;

            assert!(temp1.exists());

            fs_tx.rollback()?;

            assert!(!temp1.exists());
        }

        Ok(())
    }

    #[test]
    fn test_create_dir() -> Result<()> {
        // commit create_dir transaction
        {
            let temp1 = TempFile::new();

            let mut fs_tx = FsTransaction::new();
            fs_tx.create_dir(&temp1.path)?;
            fs_tx.commit()?;

            assert!(temp1.exists());
            assert!(dir_exists(&temp1.path)?);
        }

        // revert create_dir transaction
        {
            let temp1 = TempFile::new();

            let mut fs_tx = FsTransaction::new();
            fs_tx.create_dir(&temp1.path)?;

            fs_tx.rollback()?;

            assert!(!temp1.exists());
        }

        Ok(())
    }

    #[test]
    fn create_dir_if_missing_is_transactional_noop_for_existing_dirs() -> Result<()> {
        let existing = TempFile::new();
        existing.mkdir()?;
        let missing = TempFile::new();

        let mut fs_tx = FsTransaction::new();
        fs_tx.create_dir_if_missing(existing.as_ref())?;
        fs_tx.create_dir_if_missing(missing.as_ref())?;

        assert!(dir_exists(existing.as_ref())?);
        assert!(dir_exists(missing.as_ref())?);

        fs_tx.rollback()?;

        assert!(dir_exists(existing.as_ref())?);
        assert!(!missing.exists());

        Ok(())
    }

    #[test]
    fn create_dir_rolls_back_after_parent_sync_failure() -> Result<()> {
        let temp = TempFile::new();

        let mut fs_tx = FsTransaction::new();
        fail_parent_dir_sync_after(0);
        let err = fs_tx
            .create_dir(temp.as_ref())
            .expect_err("parent directory sync failure should fail directory creation");
        clear_parent_dir_sync_failure();

        assert!(format!("{err:?}").contains("injected parent directory sync failure"));
        assert!(dir_exists(temp.as_ref())?);

        fs_tx.rollback()?;

        assert!(!temp.exists());

        Ok(())
    }

    #[test]
    fn test_append_file() -> Result<()> {
        // commit append_file transaction
        {
            let temp1 = TempFile::new();
            temp1.write_str("foo")?;

            let mut fs_tx = FsTransaction::new();
            fs_tx.append_file(&temp1.path, "bar".as_bytes())?;
            fs_tx.commit()?;

            assert!(temp1.exists());
            assert_eq!(temp1.str_contents()?, "foobar");
        }

        // revert append_file transaction
        {
            let temp1 = TempFile::new();
            temp1.write_str("foo")?;

            let mut fs_tx = FsTransaction::new();
            fs_tx.append_file(&temp1.path, "bar".as_bytes())?;

            fs_tx.rollback()?;

            assert!(temp1.exists());
            assert_eq!(temp1.str_contents()?, "foo");
        }

        Ok(())
    }
}
