use core::fmt::Write;
use std::fs::File;
use std::io::{prelude::*, BufReader};

use anyhow::*;
use sha2::{Digest, Sha256};

pub fn get_file_hash_sha256(filepath: &str) -> Result<String> {
    let mut hasher = Sha256::new();
    let file = File::open(filepath)?;
    let mut reader = BufReader::new(file);
    let mut buffer = [0; 1024 * 1024]; // 1Mb cache

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }

        hasher.update(&buffer[..count]);
    }

    let hash = hasher.finalize();

    Ok(bytes_to_hex_string(&hash))
}

#[must_use]
pub fn bytes_to_hex_string(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(2 * bytes.len());
    for byte in bytes {
        write!(result, "{:02X}", byte).expect("failed to write data into String");
    }

    result
}

#[must_use]
pub fn gen_uuid() -> String {
    uuid::Uuid::new_v4().to_hyphenated().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_relpath;

    #[test]
    fn test_get_file_hash_sha256() -> Result<()> {
        let src = &project_relpath("../resources/k2.jpg");

        assert_eq!(
            get_file_hash_sha256(src)?,
            "1D26F4EC397E08292746D325A46D2F7A048F2840455C679EA19A85ECFA5470C9"
        );

        Ok(())
    }
}
