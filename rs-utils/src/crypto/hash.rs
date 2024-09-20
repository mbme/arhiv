use std::io::Read;

use anyhow::Result;
use sha2::{Digest, Sha256};

use crate::bytes_to_hex_string;

pub fn get_file_hash_blake3(mut reader: impl Read) -> Result<Vec<u8>> {
    let mut hasher = blake3::Hasher::new();

    let mut buffer = [0; 1024 * 1024]; // 1Mb cache

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }

        hasher.update(&buffer[..count]);
    }

    let hash = hasher.finalize();

    Ok(hash.as_bytes().to_vec())
}

pub fn get_file_hash_sha256(mut reader: impl Read) -> Result<Vec<u8>> {
    let mut hasher = Sha256::new();

    let mut buffer = [0; 1024 * 1024]; // 1Mb cache

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }

        hasher.update(&buffer[..count]);
    }

    let hash = hasher.finalize();

    Ok(hash.to_vec())
}

#[must_use]
pub fn get_string_hash_sha256(data: &str) -> String {
    let mut hasher = Sha256::new();

    hasher.update(data.as_bytes());

    let hash = hasher.finalize();

    bytes_to_hex_string(&hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bytes_to_hex_string, create_file_reader, workspace_relpath};

    #[test]
    fn test_get_file_hash_blake3() -> Result<()> {
        let src = &workspace_relpath("resources/k2.jpg");
        let reader = create_file_reader(src)?;

        assert_eq!(
            bytes_to_hex_string(&get_file_hash_blake3(reader)?),
            "33853BF0E88A13956014F28000EA7E6A8D362178E79ADAF3098F3F0B29D60301"
        );

        Ok(())
    }

    #[test]
    fn test_get_file_hash_sha256() -> Result<()> {
        let src = &workspace_relpath("resources/k2.jpg");

        let reader = create_file_reader(src)?;

        assert_eq!(
            bytes_to_hex_string(&get_file_hash_sha256(reader)?),
            "1D26F4EC397E08292746D325A46D2F7A048F2840455C679EA19A85ECFA5470C9"
        );

        Ok(())
    }

    #[test]
    fn test_get_string_hash_sha256() {
        assert_eq!(
            get_string_hash_sha256("test"),
            "9F86D081884C7D659A2FEAA0C55AD015A3BF4F1B2B0B822CD15D6C15B0F00A08"
        );
    }
}
