//! Durable single-operation filesystem mutations.
//!
//! These helpers sync written files and parent directories so callers can build
//! higher-level recovery behavior without duplicating platform-sensitive fsync
//! and cross-device rename handling.

use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

#[cfg(test)]
use std::cell::Cell;

use anyhow::{Context, Result};

use crate::{generate_alpanumeric_string, log};

/// Stages a single-file replacement through a same-directory temporary file.
///
/// Write replacement contents through `Write`, then call `commit()` to sync the
/// temporary file, publish it into place, and sync the containing directory.
/// Dropping without commit removes the temporary file on a best-effort basis.
pub struct AtomicFileWriter {
    file: fs::File,
    temp_path: PathBuf,
    dest_path: PathBuf,
    committed: bool,
}

impl AtomicFileWriter {
    /// Creates a same-directory temporary file for replacing `dest_path`.
    pub fn create(dest_path: impl AsRef<Path>) -> Result<Self> {
        let dest_path = dest_path.as_ref().to_path_buf();
        let temp_path = backup_path_with_suffix(&dest_path, "tmp");

        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .with_context(|| format!("Failed to create temp file {}", path_display(&temp_path)))?;

        Ok(Self {
            file,
            temp_path,
            dest_path,
            committed: false,
        })
    }

    /// Publishes the staged file as the destination path.
    pub fn commit(mut self) -> Result<()> {
        self.file.sync_all().with_context(|| {
            format!("Failed to sync temp file {}", path_display(&self.temp_path))
        })?;

        publish_replacement(&self.temp_path, &self.dest_path)?;
        sync_parent_dir(&self.dest_path)?;

        self.committed = true;
        Ok(())
    }
}

impl Write for AtomicFileWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl Drop for AtomicFileWriter {
    fn drop(&mut self) {
        if !self.committed
            && let Err(err) = fs::remove_file(&self.temp_path)
        {
            log::warn!(
                "Failed to remove uncommitted temp file {}: {}",
                path_display(&self.temp_path),
                err
            );
        }
    }
}

/// Writes a replacement file through a same-directory temporary path and atomic publish.
///
/// The temporary file is synced before publish and the containing directory is
/// synced after publish. This helper is for single-file durable replacement;
/// use `FsTransaction` when later steps need an in-process rollback guard.
pub fn replace_file_atomically(path: impl AsRef<Path>, data: &[u8]) -> Result<()> {
    let mut writer = AtomicFileWriter::create(path)?;
    writer
        .write_all(data)
        .context("Failed to write replacement file")?;
    writer.commit()
}

#[cfg(not(windows))]
fn publish_replacement(temp_path: &Path, dest_path: &Path) -> Result<()> {
    fs::rename(temp_path, dest_path).with_context(|| {
        format!(
            "Failed to publish replacement {} to {}",
            path_display(temp_path),
            path_display(dest_path)
        )
    })
}

#[cfg(windows)]
fn publish_replacement(temp_path: &Path, dest_path: &Path) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;

    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };

    fn wide_null(path: &Path) -> Result<Vec<u16>> {
        let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
        if wide.contains(&0) {
            anyhow::bail!("Path contains an interior NUL: {}", path_display(path));
        }
        wide.push(0);
        Ok(wide)
    }

    let temp_wide = wide_null(temp_path)?;
    let dest_wide = wide_null(dest_path)?;
    let flags = MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH;

    // MoveFileExW with REPLACE_EXISTING preserves the single publish point on
    // Windows, where std::fs::rename fails when the destination already exists.
    let moved = unsafe { MoveFileExW(temp_wide.as_ptr(), dest_wide.as_ptr(), flags) };
    if moved == 0 {
        return Err(io::Error::last_os_error()).with_context(|| {
            format!(
                "Failed to publish replacement {} to {}",
                path_display(temp_path),
                path_display(dest_path)
            )
        });
    }

    Ok(())
}

/// Copies into a newly-created file and syncs its contents.
///
/// The caller syncs the parent directory after recording any rollback state
/// needed to recover from that metadata sync failing.
pub(super) fn copy_to_new_file(reader: &mut impl Read, dest: impl AsRef<Path>) -> Result<()> {
    let dest = dest.as_ref();
    let mut writer = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dest)
        .with_context(|| format!("Failed to create Copy destination {}", path_display(dest)))?;

    if let Err(copy_err) = io::copy(reader, &mut writer).and_then(|_| writer.sync_all()) {
        drop(writer);
        if let Err(cleanup_err) = fs::remove_file(dest) {
            return Err(copy_err).with_context(|| {
                format!(
                    "Failed to remove partial Copy destination {}: {cleanup_err}",
                    path_display(dest)
                )
            });
        }

        return Err(copy_err).context("Failed to write Copy destination");
    }

    Ok(())
}

/// Moves a file and syncs the affected directories.
///
/// Falls back to copy-and-remove when `rename` crosses filesystems.
pub(super) fn move_file_durable(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    let src = src.as_ref();
    let dest = dest.as_ref();

    match fs::rename(src, dest) {
        Err(err) if err.raw_os_error() == Some(18) => {
            let mut reader = fs::File::open(src)
                .with_context(|| format!("Failed to open Move source {}", path_display(src)))?;
            copy_to_new_file(&mut reader, dest)?;
            sync_parent_dir(dest)?;
            remove_file_durable(src)?;
        }
        Err(err) => {
            return Err(err).with_context(|| {
                format!(
                    "Failed to rename {} to {}",
                    path_display(src),
                    path_display(dest)
                )
            });
        }
        Ok(()) => {
            sync_parent_dir(src)?;
            sync_parent_dir(dest)?;
        }
    }

    Ok(())
}

pub(super) fn remove_file_durable(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    remove_file_unsynced(path)?;
    sync_parent_dir(path)?;
    Ok(())
}

pub(super) fn remove_file_unsynced(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    fs::remove_file(path)
        .with_context(|| format!("Failed to remove file {}", path_display(path)))?;
    Ok(())
}

/// Syncs the containing directory so metadata updates such as rename/remove are durable.
pub(super) fn sync_parent_dir(path: impl AsRef<Path>) -> io::Result<()> {
    #[cfg(test)]
    if should_fail_parent_dir_sync() {
        return Err(io::Error::other("injected parent directory sync failure"));
    }

    #[cfg(unix)]
    if let Some(parent) = durable_parent_dir(path.as_ref()) {
        fs::File::open(parent)?.sync_all()?;
    }
    Ok(())
}

#[cfg(unix)]
fn durable_parent_dir(path: &Path) -> Option<&Path> {
    path.parent().map(|parent| {
        if parent.as_os_str().is_empty() {
            Path::new(".")
        } else {
            parent
        }
    })
}

#[cfg(test)]
thread_local! {
    static FAIL_PARENT_DIR_SYNC_AFTER: Cell<Option<usize>> = const { Cell::new(None) };
}

#[cfg(test)]
pub(super) fn fail_parent_dir_sync_after(successful_calls: usize) {
    FAIL_PARENT_DIR_SYNC_AFTER.with(|remaining| remaining.set(Some(successful_calls)));
}

#[cfg(test)]
pub(super) fn clear_parent_dir_sync_failure() {
    FAIL_PARENT_DIR_SYNC_AFTER.with(|remaining| remaining.set(None));
}

#[cfg(test)]
fn should_fail_parent_dir_sync() -> bool {
    FAIL_PARENT_DIR_SYNC_AFTER.with(|remaining| match remaining.get() {
        Some(0) => {
            remaining.set(None);
            true
        }
        Some(calls) => {
            remaining.set(Some(calls - 1));
            false
        }
        None => false,
    })
}

pub(super) fn backup_path(path: &Path) -> PathBuf {
    backup_path_with_suffix(path, "backup")
}

fn backup_path_with_suffix(path: &Path, suffix: &str) -> PathBuf {
    PathBuf::from(format!(
        "{}-{}-{suffix}",
        path_display(path),
        generate_alpanumeric_string(10)
    ))
}

pub(super) fn path_display(path: impl AsRef<Path>) -> String {
    path.as_ref().to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[cfg(unix)]
    #[test]
    fn durable_parent_dir_uses_current_directory_for_bare_relative_paths() {
        assert_eq!(
            durable_parent_dir(Path::new("storage.age")),
            Some(Path::new("."))
        );
        assert_eq!(
            durable_parent_dir(Path::new("dir/storage.age")),
            Some(Path::new("dir"))
        );
        assert_eq!(
            durable_parent_dir(Path::new("/tmp/storage.age")),
            Some(Path::new("/tmp"))
        );
    }
}
