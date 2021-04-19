use crate::log::{debug, error, warn};
use anyhow::*;
use std::fs;

enum FsOperation {
    Move { src: String, dest: String },
    Copy { src: String, dest: String },
    HardLink { src: String, dest: String },
    Remove { src: String },
}

pub struct FsTransaction {
    ops: Vec<FsOperation>,
    finished: bool,
}

impl FsTransaction {
    pub fn new() -> FsTransaction {
        FsTransaction {
            ops: vec![],
            finished: false,
        }
    }

    pub fn move_file(&mut self, src: String, dest: String) -> Result<()> {
        if let Err(err) = fs::rename(&src, &dest) {
            Err(anyhow!("Failed to Move {} to {}: {}", &src, &dest, err))
        } else {
            debug!("Moved {} to {}", &src, &dest);
            self.ops.push(FsOperation::Move { src, dest });

            Ok(())
        }
    }

    pub fn copy_file(&mut self, src: String, dest: String) -> Result<()> {
        if let Err(err) = fs::copy(&src, &dest) {
            Err(anyhow!("Failed to Copy {} to {}: {}", &src, &dest, err))
        } else {
            debug!("Copied {} to {}", &src, &dest);
            self.ops.push(FsOperation::Copy { src, dest });

            Ok(())
        }
    }

    pub fn hard_link_file(&mut self, src: String, dest: String) -> Result<()> {
        if let Err(err) = fs::hard_link(&src, &dest) {
            Err(anyhow!("Failed to HardLink {} to {}: {}", &src, &dest, err))
        } else {
            debug!("Hard Linked {} to {}", &src, &dest);
            self.ops.push(FsOperation::HardLink { src, dest });

            Ok(())
        }
    }

    pub fn remove_file(&mut self, src: String) {
        self.ops.push(FsOperation::Remove { src });
    }

    pub fn rollback(&mut self) -> Result<()> {
        if self.finished {
            return Ok(());
        }

        self.finished = true;

        if self.ops.is_empty() {
            return Ok(());
        }

        warn!("Reverting {} operations", &self.ops.len());

        let mut failed_count = 0;
        let total_count = self.ops.len();

        for op in &self.ops {
            match op {
                FsOperation::Move { src, dest } => {
                    if let Err(err) = fs::rename(dest, src) {
                        error!("Failed to revert Move {} to {}: {}", src, dest, err);
                        failed_count += 1;
                    } else {
                        warn!("Reverted Move {} to {}", src, dest);
                    }
                }

                FsOperation::Copy { src, dest } => {
                    if let Err(err) = fs::remove_file(dest) {
                        error!("Failed to revert Copy {} to {}: {}", src, dest, err);
                        failed_count += 1;
                    } else {
                        warn!("Reverted Copy {} to {}", src, dest);
                    }
                }

                FsOperation::HardLink { src, dest } => {
                    if let Err(err) = fs::remove_file(dest) {
                        error!("Failed to revert HardLink {} to {}: {}", src, dest, err);
                        failed_count += 1;
                    } else {
                        warn!("Reverted HardLink {} to {}", src, dest);
                    }
                }

                FsOperation::Remove { .. } => {
                    // do nothing
                }
            }
        }

        self.ops.clear();

        if failed_count > 0 {
            bail!(
                "Failed to revert {} operation(s) out of {}",
                failed_count,
                total_count
            )
        } else {
            Ok(())
        }
    }

    pub fn commit(&mut self) -> Result<()> {
        ensure!(
            !self.finished,
            "must not try to commit finished transaction"
        );

        self.finished = true;

        for op in self.ops.iter() {
            if let FsOperation::Remove { src } = op {
                if let Err(err) = fs::remove_file(src) {
                    error!("Failed to commit Remove {} : {}", src, err);
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
