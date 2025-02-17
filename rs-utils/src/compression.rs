use std::io::{BufRead, BufReader, Read, Write};

use anyhow::Result;
use flate2::{bufread::GzDecoder, write::GzEncoder, Compression};

use crate::{
    age::{AgeWriter, PublicKey},
    confidential1::{Confidential1Key, Confidential1Reader, Confidential1Writer},
    crypto::age::{AgeReader, PrivateKey},
};

pub fn create_gz_reader<R: BufRead>(reader: R) -> GzDecoder<R> {
    GzDecoder::new(reader)
}

pub fn create_gz_writer<W: Write>(writer: W) -> GzEncoder<W> {
    GzEncoder::new(writer, Compression::fast())
}

pub struct AgeGzReader<R: Read> {
    inner: GzDecoder<BufReader<AgeReader<R>>>,
}

impl<R: Read> AgeGzReader<R> {
    pub fn create(reader: R, key: PrivateKey) -> Result<Self> {
        let age_reader = AgeReader::new(reader, key)?;
        let age_buf_reader = BufReader::new(age_reader);

        let gz_reader = create_gz_reader(age_buf_reader);

        Ok(Self { inner: gz_reader })
    }
}

impl<R: Read> Read for AgeGzReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

pub struct AgeGzWriter<W: Write> {
    inner: GzEncoder<AgeWriter<W>>,
}

impl<W: Write> AgeGzWriter<W> {
    pub fn create(writer: W, key: PublicKey) -> Result<Self> {
        let age_writer = AgeWriter::new(writer, key)?;
        let gz_writer = create_gz_writer(age_writer);

        Ok(Self { inner: gz_writer })
    }

    /// must be called
    pub fn finish(self) -> Result<W> {
        let age_writer = self.inner.finish()?;

        age_writer.finish()
    }
}

impl<W: Write> Write for AgeGzWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

pub struct C1GzReader<R: Read> {
    inner: GzDecoder<BufReader<Confidential1Reader<R>>>,
}

impl<R: Read> C1GzReader<R> {
    pub fn create(reader: R, key: &Confidential1Key) -> Result<Self> {
        let c1_reader = Confidential1Reader::new(reader, key)?;
        let c1_buf_reader = BufReader::new(c1_reader);

        let gz_reader = create_gz_reader(c1_buf_reader);

        Ok(Self { inner: gz_reader })
    }
}

impl<R: Read> Read for C1GzReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

pub struct C1GzWriter<W: Write> {
    inner: GzEncoder<Confidential1Writer<W>>,
}

impl<W: Write> C1GzWriter<W> {
    pub fn create(writer: W, key: &Confidential1Key) -> Result<Self> {
        let c1_writer = Confidential1Writer::new(writer, key)?;
        let gz_writer = create_gz_writer(c1_writer);

        Ok(Self { inner: gz_writer })
    }

    /// must be called
    pub fn finish(self) -> Result<W> {
        let c1_writer = self.inner.finish()?;

        c1_writer.finish()
    }
}

impl<W: Write> Write for C1GzWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::{
        age::{PrivateKey, PublicKey},
        confidential1::Confidential1Key,
        generate_alpanumeric_string, read_all_as_string, AgeGzReader, AgeGzWriter, C1GzReader,
        C1GzWriter,
    };

    use super::{create_gz_reader, create_gz_writer};

    #[test]
    fn test_gz_read_write() {
        let data = generate_alpanumeric_string(100);

        let mut writer = create_gz_writer(Vec::new());
        writer.write_all(data.as_bytes()).unwrap();

        let compressed = writer.finish().unwrap();

        let reader = create_gz_reader(compressed.as_slice());
        let result = read_all_as_string(reader).unwrap();

        assert_eq!(result, data);
    }

    #[test]
    fn test_c1_gz_read_write() {
        let data = generate_alpanumeric_string(100);

        let c1key = Confidential1Key::new_random_key();
        let mut writer = C1GzWriter::create(Vec::new(), &c1key).unwrap();
        writer.write_all(data.as_bytes()).unwrap();

        let compressed = writer.finish().unwrap();

        let reader = C1GzReader::create(compressed.as_slice(), &c1key).unwrap();
        let result = read_all_as_string(reader).unwrap();

        assert_eq!(result, data);
    }

    #[test]
    fn test_age_gz_read_write() {
        let data = generate_alpanumeric_string(100);

        let key = PrivateKey::generate_age_x25519_key();
        let private_key = PrivateKey::from_age_x25519_key(key.duplicate()).unwrap();
        let public_key = PublicKey::from_age_x25519_key(key.duplicate()).unwrap();

        let mut writer = AgeGzWriter::create(Vec::new(), public_key).unwrap();
        writer.write_all(data.as_bytes()).unwrap();

        let compressed = writer.finish().unwrap();

        let reader = AgeGzReader::create(compressed.as_slice(), private_key).unwrap();
        let result = read_all_as_string(reader).unwrap();

        assert_eq!(result, data);
    }
}
