use anyhow::*;
use std::fs;

pub fn ensure_exists(path: &str, dir: bool) -> Result<()> {
    match fs::metadata(path) {
        Ok(metadata) if dir && !metadata.is_dir() => {
            return Err(anyhow!("path isn't a directory: {}", path));
        }

        Ok(metadata) if !dir && !metadata.is_file() => {
            return Err(anyhow!("path isn't a file: {}", path));
        }

        Ok(_) => Ok(()),

        Err(_) => Err(anyhow!("path doesn't exist {}", path)),
    }
}
