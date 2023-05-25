use std::{fs, io::Write};

use anyhow::{anyhow, bail, ensure, Context, Result};

use crate::{
    ensure_file_exists, generate_alpanumeric_string, get_file_size, log, move_file,
    must_create_file, path_exists, set_file_size,
};

enum FsOperation {
    Backup { src: String, dest: String },
    Move { src: String, dest: String },
    Copy { src: String, dest: String },
    HardLink { src: String, dest: String },
    CreateFile { path: String },
    CreateDir { path: String },
    AppendFile { path: String, original_size: u64 },
}

pub struct FsTransaction {
    ops: Vec<FsOperation>,
}

// TODO make sure few transactions work in parallel on the same files
// Works for files but not for dirs
impl FsTransaction {
    #[must_use]
    pub fn new() -> FsTransaction {
        FsTransaction { ops: vec![] }
    }

    fn move_to_backup(&mut self, src: impl Into<String>) -> Result<()> {
        let src = src.into();
        let dest = format!("{}-{}-backup", src, generate_alpanumeric_string(10));

        ensure!(!path_exists(&dest), "backup path must not exist");

        if let Err(err) = move_file(&src, &dest) {
            bail!("Failed to Backup {} to {}: {}", &src, &dest, err);
        }

        log::debug!("Backed up {} to {}", &src, &dest);
        self.ops.push(FsOperation::Backup { src, dest });

        Ok(())
    }

    pub fn move_file(&mut self, src: impl Into<String>, dest: impl Into<String>) -> Result<()> {
        let src = src.into();
        let dest = dest.into();

        if path_exists(&dest) {
            self.move_to_backup(&dest)?;
        }

        if let Err(err) = move_file(&src, &dest) {
            bail!("Failed to Move {} to {}: {}", &src, &dest, err);
        }

        log::debug!("Moved {} to {}", &src, &dest);
        self.ops.push(FsOperation::Move { src, dest });

        Ok(())
    }

    pub fn copy_file(&mut self, src: impl Into<String>, dest: impl Into<String>) -> Result<()> {
        let src = src.into();
        let dest = dest.into();

        if path_exists(&dest) {
            self.move_to_backup(&dest)?;
        }

        if let Err(err) = fs::copy(&src, &dest) {
            bail!("Failed to Copy {} to {}: {}", &src, &dest, err);
        }

        log::debug!("Copied {} to {}", &src, &dest);
        self.ops.push(FsOperation::Copy { src, dest });

        Ok(())
    }

    pub fn hard_link_file(
        &mut self,
        src: impl Into<String>,
        dest: impl Into<String>,
    ) -> Result<()> {
        let src = src.into();
        let dest = dest.into();

        if path_exists(&dest) {
            self.move_to_backup(&dest)?;
        }

        if let Err(err) = fs::hard_link(&src, &dest) {
            bail!("Failed to HardLink {} to {}: {}", &src, &dest, err);
        }

        log::debug!("Hard Linked {} to {}", &src, &dest);
        self.ops.push(FsOperation::HardLink { src, dest });

        Ok(())
    }

    pub fn remove_file(&mut self, src: impl Into<String>) -> Result<()> {
        let src = src.into();

        self.move_to_backup(&src)?;

        log::debug!("Removed file {}", &src);

        Ok(())
    }

    pub fn create_file(&mut self, path: impl Into<String>, data: &[u8]) -> Result<()> {
        let path = path.into();

        let mut file = must_create_file(&path).context(anyhow!("Failed to Create file {path}"))?;

        if !data.is_empty() {
            file.write_all(data)
                .context(anyhow!("Failed to write data into file {path}"))?;
        }

        file.sync_all()
            .context("Failed to sync file changes to disk")?;

        log::debug!("Created file {path}");

        self.ops.push(FsOperation::CreateFile { path });

        Ok(())
    }

    pub fn create_dir(&mut self, path: impl Into<String>) -> Result<()> {
        let path = path.into();

        fs::create_dir(&path).context(anyhow!("Failed to Create dir {path}"))?;

        log::debug!("Created dir {path}");

        self.ops.push(FsOperation::CreateDir { path });

        Ok(())
    }

    pub fn append_file(&mut self, path: impl Into<String>, data: &[u8]) -> Result<()> {
        let path = path.into();

        ensure_file_exists(&path)?;

        let original_size = get_file_size(&path)?;

        let mut file = fs::OpenOptions::new()
            .read(false)
            .append(true)
            .create_new(false)
            .open(&path)
            .context(anyhow!("Failed to open file {path}"))?;

        file.write_all(data)
            .context(anyhow!("Failed to append data to file {path}"))?;

        file.sync_all()
            .context("Failed to sync file changes to disk")?;

        log::debug!("Appended {} bytes to file {path}", data.len());

        self.ops.push(FsOperation::AppendFile {
            path,
            original_size,
        });

        Ok(())
    }

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
                    if let Err(err) = move_file(dest, src) {
                        log::error!("Failed to revert Move {} to {}: {}", src, dest, err);
                        failed_count += 1;
                    } else {
                        log::warn!("Reverted Move {} to {}", src, dest);
                    }
                }

                FsOperation::Copy { src, dest } => {
                    if let Err(err) = fs::remove_file(dest) {
                        log::error!("Failed to revert Copy {} to {}: {}", src, dest, err);
                        failed_count += 1;
                    } else {
                        log::warn!("Reverted Copy {} to {}", src, dest);
                    }
                }

                FsOperation::HardLink { src, dest } => {
                    if let Err(err) = fs::remove_file(dest) {
                        log::error!("Failed to revert HardLink {} to {}: {}", src, dest, err);
                        failed_count += 1;
                    } else {
                        log::warn!("Reverted HardLink {} to {}", src, dest);
                    }
                }

                FsOperation::Backup { src, dest } => {
                    if let Err(err) = move_file(dest, src) {
                        log::error!("Failed to revert Backup {} to {}: {}", src, dest, err);
                        failed_count += 1;
                    } else {
                        log::warn!("Reverted Backup {} to {}", src, dest);
                    }
                }
                FsOperation::CreateFile { path } => {
                    if let Err(err) = fs::remove_file(path) {
                        log::error!("Failed to revert CreateFile {path}: {err}");
                        failed_count += 1;
                    } else {
                        log::warn!("Reverted CreateFile {path}");
                    }
                }
                FsOperation::CreateDir { path } => {
                    if let Err(err) = fs::remove_dir(path) {
                        log::error!("Failed to revert CreateDir {path}: {err}");
                        failed_count += 1;
                    } else {
                        log::warn!("Reverted CreateDir {path}");
                    }
                }
                FsOperation::AppendFile {
                    path,
                    original_size,
                } => {
                    if let Err(err) = set_file_size(path, *original_size) {
                        log::error!("Failed to revert AppendFile {path}: {err}");
                        failed_count += 1;
                    } else {
                        log::warn!("Reverted AppendFile {path}");
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

    pub fn commit(&mut self) -> Result<()> {
        for op in &self.ops {
            if let FsOperation::Backup { dest, .. } = op {
                if let Err(err) = fs::remove_file(dest) {
                    log::error!("Failed to remove Backup {} : {}", dest, err);
                }
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
    use crate::{dir_exists, TempFile};

    use super::*;

    #[test]
    fn test_move() -> Result<()> {
        // commit move transaction
        {
            let temp1 = TempFile::new();
            temp1.write_str("temp1")?;

            let temp2 = TempFile::new();

            let mut fs_tx = FsTransaction::new();
            fs_tx.move_file(temp1.as_ref(), temp2.as_ref())?;
            fs_tx.commit()?;

            assert!(!temp1.exists());
            assert!(temp2.exists());
            assert_eq!(temp2.str_contents()?, "temp1");
        }

        // revert move transaction & restore backup
        {
            let temp1 = TempFile::new();
            temp1.write_str("temp1")?;

            let temp2 = TempFile::new();
            temp2.write_str("temp2")?;

            let mut fs_tx = FsTransaction::new();
            fs_tx.move_file(temp1.as_ref(), temp2.as_ref())?;

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
