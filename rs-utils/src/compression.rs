use std::io::{BufRead, BufReader, Read, Write};

use anyhow::Result;
use flate2::{bufread::GzDecoder, write::GzEncoder, Compression};

use crate::confidential1::{Confidential1Key, Confidential1Reader, Confidential1Writer};

pub fn create_gz_reader<R: BufRead>(reader: R) -> GzDecoder<R> {
    GzDecoder::new(reader)
}

pub fn create_gz_writer<W: Write>(writer: W) -> GzEncoder<W> {
    GzEncoder::new(writer, Compression::fast())
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

    use anyhow::Result;

    use crate::{
        confidential1::Confidential1Key, generate_alpanumeric_string, read_all_as_string,
        C1GzReader, C1GzWriter,
    };

    use super::{create_gz_reader, create_gz_writer};

    #[test]
    fn test_gz_read_write() -> Result<()> {
        let data = generate_alpanumeric_string(100);

        let mut writer = create_gz_writer(Vec::new());
        writer.write_all(data.as_bytes())?;

        let compressed = writer.finish()?;

        let reader = create_gz_reader(compressed.as_slice());
        let result = read_all_as_string(reader)?;

        assert_eq!(result, data);

        Ok(())
    }

    #[test]
    fn test_c1_gz_read_write() -> Result<()> {
        let data = generate_alpanumeric_string(100);

        let c1key = Confidential1Key::new_random_key();
        let mut writer = C1GzWriter::create(Vec::new(), &c1key)?;
        writer.write_all(data.as_bytes())?;

        let compressed = writer.finish()?;

        let reader = C1GzReader::create(compressed.as_slice(), &c1key)?;
        let result = read_all_as_string(reader)?;

        assert_eq!(result, data);

        Ok(())
    }
}
