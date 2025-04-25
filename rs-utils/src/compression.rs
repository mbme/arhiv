use std::io::{BufRead, BufReader, Read, Write};

use anyhow::Result;
use flate2::{bufread::GzDecoder, write::GzEncoder, Compression};

use crate::age::{AgeKey, AgeReader, AgeWriter};

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
    pub fn new(reader: R, key: AgeKey) -> Result<Self> {
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
    pub fn new(writer: W, key: AgeKey) -> Result<Self> {
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

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::{
        age::AgeKey, generate_alpanumeric_string, read_all_as_string, AgeGzReader, AgeGzWriter,
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
    fn test_age_gz_read_write() {
        let data = generate_alpanumeric_string(100);

        let key = AgeKey::generate_age_x25519_key();

        let mut writer = AgeGzWriter::new(Vec::new(), key.clone()).unwrap();
        writer.write_all(data.as_bytes()).unwrap();

        let compressed = writer.finish().unwrap();

        let reader = AgeGzReader::new(compressed.as_slice(), key).unwrap();
        let result = read_all_as_string(reader).unwrap();

        assert_eq!(result, data);
    }
}
