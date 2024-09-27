use std::io::{Read, Write};

use anyhow::Result;
use sha2::{Digest, Sha256};

use crate::bytes_to_hex_string;

pub const SHA256_HASH_SIZE: usize = 32;
pub type Sha256Hash = [u8; SHA256_HASH_SIZE];

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

pub fn get_file_hash_sha256(mut reader: impl Read) -> Result<Sha256Hash> {
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

    Ok(hash.into())
}

#[must_use]
pub fn get_string_hash_sha256(data: &str) -> String {
    let mut hasher = Sha256::new();

    hasher.update(data.as_bytes());

    let hash = hasher.finalize();

    bytes_to_hex_string(&hash)
}

enum HasherOrHash {
    Hasher(Sha256),
    Hash(Sha256Hash),
    Undefined,
}

pub struct Sha256HashingReader<R: Read> {
    inner: R,
    hasher: HasherOrHash,
}

impl<R: Read> Sha256HashingReader<R> {
    pub fn new(inner: R) -> Self {
        let hasher = HasherOrHash::Hasher(Sha256::new());

        Self { inner, hasher }
    }

    pub fn get_hash(&self) -> Option<&Sha256Hash> {
        if let HasherOrHash::Hash(ref hash) = self.hasher {
            Some(hash)
        } else {
            None
        }
    }

    pub fn get_inner(&self) -> &R {
        &self.inner
    }

    fn update_hasher(&mut self, buf: &[u8]) {
        let mut hasher = HasherOrHash::Undefined;
        std::mem::swap(&mut self.hasher, &mut hasher);

        self.hasher = match hasher {
            HasherOrHash::Hasher(mut hasher) => {
                if buf.is_empty() {
                    let hash = hasher.finalize().into();
                    HasherOrHash::Hash(hash)
                } else {
                    hasher.update(buf);
                    HasherOrHash::Hasher(hasher)
                }
            }
            HasherOrHash::Hash(_) => return,
            HasherOrHash::Undefined => panic!("self.hasher was Undefined"),
        };
    }
}

impl<R: Read> Read for Sha256HashingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_read = self.inner.read(buf)?;

        self.update_hasher(&buf[0..bytes_read]);

        Ok(bytes_read)
    }
}

pub struct Sha256HashingWriter<W: Write> {
    inner: W,
    hasher: Sha256,
}

impl<W: Write> Sha256HashingWriter<W> {
    pub fn new(inner: W) -> Self {
        let hasher = Sha256::new();

        Self { inner, hasher }
    }

    pub fn finish(self) -> (Sha256Hash, W) {
        let hash = self.hasher.finalize().into();

        (hash, self.inner)
    }
}

impl<W: Write> Write for Sha256HashingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let bytes_written = self.inner.write(buf)?;

        if bytes_written > 0 {
            self.hasher.update(&buf[0..bytes_written]);
        }

        Ok(bytes_written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bytes_to_hex_string, create_file_reader, read_all_as_string, workspace_relpath};

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

    #[test]
    fn test_hashing_writer() -> Result<()> {
        let data = "test";

        let mut writer = Sha256HashingWriter::new(Vec::new());
        writer.write_all(data.as_bytes())?;
        let (hash, result) = writer.finish();

        assert_eq!(result, data.as_bytes());
        assert_eq!(hash, get_file_hash_sha256(data.as_bytes())?);

        Ok(())
    }

    #[test]
    fn test_hashing_reader() -> Result<()> {
        let data = "test";

        let mut reader = Sha256HashingReader::new(data.as_bytes());
        let result = read_all_as_string(&mut reader)?;

        assert_eq!(result, data);

        assert_eq!(
            reader.get_hash(),
            Some(&get_file_hash_sha256(data.as_bytes())?)
        );

        Ok(())
    }
}
