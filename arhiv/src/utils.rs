use anyhow::*;
use std::fs;

pub fn ensure_exists(path: &str, dir: bool) -> Result<()> {
    match fs::metadata(path) {
        Ok(metadata) if dir && !metadata.is_dir() => {
            Err(anyhow!("path isn't a directory: {}", path))
        }

        Ok(metadata) if !dir && !metadata.is_file() => Err(anyhow!("path isn't a file: {}", path)),

        Ok(_) => Ok(()),

        Err(_) => Err(anyhow!("path doesn't exist {}", path)),
    }
}

enum FsOperation {
    Move { src: String, dest: String },
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
            log::debug!("Moved {} to {}", &src, &dest);
            self.ops.push(FsOperation::Move { src, dest });

            Ok(())
        }
    }

    pub fn revert(&mut self) {
        if self.ops.is_empty() {
            return;
        }

        log::warn!("Reverting {} operations", &self.ops.len());

        for op in &self.ops {
            match op {
                FsOperation::Move { src, dest } => {
                    if let Err(err) = fs::rename(dest, src) {
                        log::error!("Failed to revert Move {} to {}: {}", src, dest, err);
                    } else {
                        log::warn!("Reverted Move {} to {}", src, dest);
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
