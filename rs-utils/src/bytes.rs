use std::io::{self, Read, Seek, SeekFrom};

use anyhow::{Context, Result};
use data_encoding::{BASE64, BASE64URL, HEXUPPER};
use rand::{rngs::OsRng, thread_rng, RngCore};

pub fn generate_bytes(n: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; n];

    let mut rng = thread_rng();

    rng.fill_bytes(&mut bytes);

    bytes
}

#[must_use]
pub fn new_random_crypto_byte_array<const SIZE: usize>() -> [u8; SIZE] {
    let mut bytes = [0u8; SIZE];

    OsRng.fill_bytes(&mut bytes);

    bytes
}

#[must_use]
pub fn to_url_safe_base64(bytes: &[u8]) -> String {
    BASE64URL.encode(bytes)
}

#[must_use]
pub fn is_valid_base64(value: &str) -> bool {
    BASE64URL.decode(value.as_bytes()).is_ok()
}

pub fn decode_base64(data: &str) -> Result<Vec<u8>> {
    BASE64
        .decode(data.as_bytes())
        .context("Failed to decode base64 string")
}

pub fn bytes_to_hex_string(bytes: &[u8]) -> String {
    HEXUPPER.encode(bytes)
}

pub fn hex_string_to_bytes(hex: &str) -> Result<Vec<u8>> {
    HEXUPPER
        .decode(hex.as_bytes())
        .context("Failed to decode hex string")
}

pub fn read_all_as_string(mut reader: impl Read) -> io::Result<String> {
    let mut result = String::new();
    reader.read_to_string(&mut result)?;

    Ok(result)
}

pub struct ReaderWithTrailer<const SIZE: usize, R: Read> {
    inner: R,
    buffer: Vec<u8>,
    trailer: Option<[u8; SIZE]>,
}

impl<const SIZE: usize, R: Read> ReaderWithTrailer<SIZE, R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            buffer: Vec::new(),
            trailer: None,
        }
    }

    pub fn get_trailer(&self) -> Option<&[u8; SIZE]> {
        self.trailer.as_ref()
    }

    pub fn has_trailer(&self) -> bool {
        self.get_trailer().is_some()
    }

    pub fn get_inner(&self) -> &R {
        &self.inner
    }
}

impl<const SIZE: usize, R: Read> Read for ReaderWithTrailer<SIZE, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.buffer.is_empty() {
            self.buffer.resize(SIZE, 0);
            self.inner.read_exact(&mut self.buffer)?;
        }

        assert!(
            self.buffer.len() >= SIZE,
            "Buffer is at least as long as the trailer"
        );

        self.buffer.resize(SIZE + buf.len(), 0);

        let bytes_read = self.inner.read(&mut self.buffer[SIZE..])?;
        if bytes_read == 0 {
            self.trailer = Some(
                self.buffer[0..SIZE]
                    .try_into()
                    .expect("Buffer must have enough elements"),
            );
            return Ok(0);
        }

        buf[0..bytes_read].copy_from_slice(&self.buffer[0..bytes_read]);
        self.buffer.drain(0..bytes_read);

        Ok(bytes_read)
    }
}

impl<const SIZE: usize, R: Read + Seek> ReaderWithTrailer<SIZE, R> {
    pub fn read_trailer(&mut self) -> io::Result<()> {
        if self.has_trailer() {
            return Ok(());
        }

        self.seek(SeekFrom::End(-(SIZE as i64)))?;

        let mut trailer = [0; SIZE];
        self.inner.read_exact(&mut trailer)?;
        self.trailer = Some(trailer);

        Ok(())
    }
}

impl<const SIZE: usize, R: Read + Seek> Seek for ReaderWithTrailer<SIZE, R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.buffer.clear();
        self.inner.seek(pos)
    }
}

pub struct ReaderWithHeader<const SIZE: usize, R: Read> {
    inner: R,
}

impl<const SIZE: usize, R: Read> ReaderWithHeader<SIZE, R> {
    pub fn new(mut inner: R) -> io::Result<(Self, [u8; SIZE])> {
        let mut data = [0; SIZE];
        inner.read_exact(&mut data)?;

        let reader = Self { inner };

        Ok((reader, data))
    }

    pub fn get_inner(&self) -> &R {
        &self.inner
    }
}

impl<const SIZE: usize, R: Read> Read for ReaderWithHeader<SIZE, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<const SIZE: usize, R: Read + Seek> Seek for ReaderWithHeader<SIZE, R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let normalized_pos = match pos {
            SeekFrom::Start(offset) => SeekFrom::Start(offset + SIZE as u64),
            _ => pos,
        };

        let pos = self.inner.seek(normalized_pos)?;
        if pos < SIZE as u64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot seek before the start of data",
            ));
        }

        Ok(pos - SIZE as u64)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, SeekFrom};

    use rand::RngCore;

    use super::*;

    #[test]
    fn test_hex_encode_decode() {
        let mut data = [0u8; 150];
        rand::thread_rng().fill_bytes(&mut data);

        let result = bytes_to_hex_string(&data);
        let result = hex_string_to_bytes(&result).unwrap();

        assert_eq!(data, result.as_slice());
    }

    #[test]
    fn test_reader_with_trailer() -> Result<()> {
        {
            let data = generate_bytes(100);
            let mut reader = ReaderWithTrailer::<200, _>::new(data.as_slice());

            let mut result = Vec::new();
            // not enough data for a trailer
            assert!(reader.read_to_end(&mut result).is_err());
        }

        {
            let data = generate_bytes(300);
            let mut reader = ReaderWithTrailer::<200, _>::new(data.as_slice());

            let mut result = Vec::new();
            loop {
                let mut buffer = [0; 10];
                let bytes_read = reader.read(&mut buffer)?;

                if bytes_read > 0 {
                    result.extend_from_slice(&buffer[0..bytes_read]);
                } else {
                    break;
                }
            }

            assert_eq!(result.len(), 100);
            assert_eq!(result.as_slice(), &data[0..100]);
            assert_eq!(reader.get_trailer().unwrap(), &data[100..]);
        }

        {
            let data = generate_bytes(300);
            let mut reader = ReaderWithTrailer::<100, _>::new(Cursor::new(data));

            reader.seek(SeekFrom::Current(250))?;

            let mut result = Vec::new();
            // not enough data for a trailer
            assert!(reader.read_to_end(&mut result).is_err());
        }

        {
            let data = generate_bytes(300);
            let mut reader = ReaderWithTrailer::<100, _>::new(Cursor::new(&data));

            reader.seek(SeekFrom::Current(200))?;

            let mut result = Vec::new();
            reader.read_to_end(&mut result)?;

            assert_eq!(result.len(), 0);
            assert_eq!(reader.get_trailer().unwrap(), &data[200..]);
        }

        {
            let data = generate_bytes(300);
            let mut reader = ReaderWithTrailer::<100, _>::new(Cursor::new(&data));

            reader.read_trailer()?;

            assert_eq!(reader.get_trailer().unwrap(), &data[200..]);
        }

        Ok(())
    }

    #[test]
    fn test_reader_with_header() -> Result<()> {
        {
            let data = generate_bytes(100);
            // not enough data for a header
            assert!(ReaderWithHeader::<200, _>::new(data.as_slice()).is_err());
        }

        {
            let data = generate_bytes(300);
            let (mut reader, header) = ReaderWithHeader::<200, _>::new(data.as_slice())?;

            let mut result = Vec::new();
            reader.read_to_end(&mut result)?;

            assert_eq!(header, &data[0..200]);
            assert_eq!(result, &data[200..]);
        }

        {
            let data = generate_bytes(300);
            let (mut reader, _) = ReaderWithHeader::<100, _>::new(Cursor::new(&data))?;

            reader.seek(SeekFrom::Current(50))?;

            let mut result = Vec::new();
            reader.read_to_end(&mut result)?;

            assert_eq!(result, &data[150..]);
        }

        Ok(())
    }
}
