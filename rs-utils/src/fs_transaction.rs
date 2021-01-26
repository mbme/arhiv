use anyhow::*;
use std::fs;
use tracing::{debug, error, warn};

enum FsOperation {
    Move { src: String, dest: String },
    Copy { src: String, dest: String },
    HardLink { src: String, dest: String },
}

pub struct FsTransaction {
    ops: Vec<FsOperation>,
}

impl FsTransaction {
    pub fn new() -> FsTransaction {
        FsTransaction { ops: vec![] }
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

    pub fn revert(&mut self) {
        if self.ops.is_empty() {
            return;
        }

        warn!("Reverting {} operations", &self.ops.len());

        for op in &self.ops {
            match op {
                FsOperation::Move { src, dest } => {
                    if let Err(err) = fs::rename(dest, src) {
                        error!("Failed to revert Move {} to {}: {}", src, dest, err);
                    } else {
                        warn!("Reverted Move {} to {}", src, dest);
                    }
                }

                FsOperation::Copy { src, dest } => {
                    if let Err(err) = fs::remove_file(dest) {
                        error!("Failed to revert Copy {} to {}: {}", src, dest, err);
                    } else {
                        warn!("Reverted Copy {} to {}", src, dest);
                    }
                }

                FsOperation::HardLink { src, dest } => {
                    if let Err(err) = fs::remove_file(dest) {
                        error!("Failed to revert HardLink {} to {}: {}", src, dest, err);
                    } else {
                        warn!("Reverted HardLink {} to {}", src, dest);
                    }
                }
            }
        }

        self.ops.clear();
    }

    pub fn commit(&mut self) {
        self.ops.clear();
    }
}

impl Drop for FsTransaction {
    fn drop(&mut self) {
        self.revert();
    }
}
