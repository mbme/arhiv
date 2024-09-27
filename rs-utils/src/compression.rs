use std::io::{BufRead, Write};

use flate2::{bufread::GzDecoder, write::GzEncoder, Compression};

pub fn create_gz_reader<R: BufRead>(reader: R) -> GzDecoder<R> {
    GzDecoder::new(reader)
}

pub fn create_gz_writer<W: Write>(writer: W) -> GzEncoder<W> {
    GzEncoder::new(writer, Compression::fast())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use anyhow::Result;

    use crate::{generate_alpanumeric_string, read_all_as_string};

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
}
