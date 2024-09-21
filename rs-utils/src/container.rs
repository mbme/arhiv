use std::{
    borrow::Cow,
    io::{BufRead, BufReader, Write},
};

use anyhow::{ensure, Context, Result};
use flate2::{bufread::GzDecoder, write::GzEncoder, Compression};
use serde::{Deserialize, Serialize};

use crate::TakeExactly;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(transparent, deny_unknown_fields)]
pub struct LinesIndex<'a> {
    index: Vec<Cow<'a, str>>,
}

impl<'a> LinesIndex<'a> {
    #[must_use]
    pub fn len(&self) -> usize {
        self.index.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a> From<&[&'a str]> for LinesIndex<'a> {
    fn from(value: &[&'a str]) -> Self {
        let index = value.iter().map(|line| (*line).into()).collect();

        Self { index }
    }
}

impl<'a> From<&'a [String]> for LinesIndex<'a> {
    fn from(value: &'a [String]) -> Self {
        let index = value.iter().map(|value| value.as_str().into()).collect();

        Self { index }
    }
}

impl<'a> From<&'a Vec<String>> for LinesIndex<'a> {
    fn from(value: &'a Vec<String>) -> Self {
        value.as_slice().into()
    }
}

impl<'a> PartialEq<Vec<String>> for LinesIndex<'a> {
    fn eq(&self, other: &Vec<String>) -> bool {
        self.index
            .iter()
            .map(|cow| cow.as_ref())
            .eq(other.iter().map(String::as_str))
    }
}

impl<'a> PartialEq<Vec<&str>> for LinesIndex<'a> {
    fn eq(&self, other: &Vec<&str>) -> bool {
        self.index
            .iter()
            .map(|cow| cow.as_ref())
            .eq(other.iter().copied())
    }
}

pub fn create_gz_reader(reader: impl BufRead) -> impl BufRead {
    let gz_reader = GzDecoder::new(reader);

    BufReader::new(gz_reader)
}

pub fn create_gz_writer<W: Write>(writer: W) -> GzEncoder<W> {
    GzEncoder::new(writer, Compression::fast())
}

pub struct ContainerReader<'i, R: BufRead> {
    index: LinesIndex<'i>,
    reader: R,
}

impl<'i, R: BufRead> ContainerReader<'i, R> {
    pub fn init(mut reader: R) -> Result<Self> {
        let mut line = String::new();
        reader.read_line(&mut line)?;

        let index: LinesIndex =
            serde_json::from_str(&line).context("Failed to parse container index")?;

        Ok(Self { index, reader })
    }

    pub fn get_index(&self) -> &LinesIndex {
        &self.index
    }

    pub fn iter_lines(self) -> impl Iterator<Item = Result<String>> {
        let lines = self
            .reader
            .lines()
            .enumerate()
            .map(|(i, result)| result.with_context(|| format!("Failed to read line {i}")));

        TakeExactly::new(lines, self.index.len())
    }

    pub fn read_all(self) -> Result<Vec<String>> {
        self.iter_lines().collect()
    }
}

pub struct ContainerWriter<W: Write> {
    writer: W,
    lines: usize,
}

impl<W: Write> ContainerWriter<W> {
    pub fn init(mut writer: W, index: &LinesIndex) -> Result<Self> {
        // false positive clippy lint
        #[allow(clippy::needless_borrows_for_generic_args)]
        serde_json::to_writer(&mut writer, &index)?;

        Ok(Self {
            writer,
            lines: index.len(),
        })
    }

    pub fn write_lines<'a>(mut self, lines: impl Iterator<Item = &'a str>) -> Result<()> {
        let mut lines_count = 0;
        for (i, line) in lines.enumerate() {
            ensure!(i < self.lines, "Expected only {} lines", self.lines);

            lines_count += 1;

            self.writer.write_all(b"\n")?;
            self.writer.write_all(line.as_bytes())?;
        }

        self.writer.flush()?;

        ensure!(
            lines_count == self.lines,
            "Expected {} lines, got {lines_count}",
            self.lines
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use anyhow::Result;
    use rand::Rng;

    use crate::{create_gz_reader, ContainerWriter};

    use super::{create_gz_writer, ContainerReader, LinesIndex};

    #[test]
    fn test_line_index_serialization() -> Result<()> {
        let raw_index = r#"["1","2","3"]"#;
        let index: LinesIndex<'_> = serde_json::from_str(raw_index)?;

        assert_eq!(index, vec!["1", "2", "3"]);

        let serialized_index = serde_json::to_string(&index)?;

        assert_eq!(serialized_index, raw_index);

        Ok(())
    }

    #[test]
    fn test_read_container_lines() -> Result<()> {
        {
            let raw_data = r#"["1","2","3"]
1
2
3"#;
            let data = Cursor::new(raw_data.as_bytes());
            let reader = ContainerReader::init(data)?;
            assert_eq!(*reader.get_index(), vec!["1", "2", "3"]);

            let new_lines = reader.read_all()?;
            assert_eq!(new_lines, vec!["1", "2", "3"]);
        }

        {
            let raw_data = r#"["1","2","3"]
1
2"#;
            let data = Cursor::new(raw_data.as_bytes());
            let reader = ContainerReader::init(data)?;

            assert!(reader.read_all().is_err());
        }

        {
            let raw_data = r#"["1","2","3"]
1
2
3
4"#;
            let data = Cursor::new(raw_data.as_bytes());
            let reader = ContainerReader::init(data)?;

            assert!(reader.read_all().is_err());
        }

        Ok(())
    }

    #[test]
    fn test_write_container_lines() -> Result<()> {
        {
            let mut data = Cursor::new(Vec::new());

            let lines = ["3", "2", "1"];
            let index = lines.as_slice().into();
            let writer = ContainerWriter::init(&mut data, &index)?;
            writer.write_lines(lines.into_iter())?;

            let data = String::from_utf8(data.into_inner())?;

            assert_eq!(
                data,
                r#"["3","2","1"]
3
2
1"#
            );
        }

        {
            let mut data = Cursor::new(Vec::new());

            let index = ["1", "2"].as_slice().into();
            let lines = ["1", "2", "3"];
            let writer = ContainerWriter::init(&mut data, &index)?;
            assert!(writer.write_lines(lines.into_iter()).is_err());

            let data = String::from_utf8(data.into_inner())?;

            assert_eq!(
                data,
                r#"["1","2"]
1
2"#
            );
        }

        {
            let mut data = Cursor::new(Vec::new());

            let index = ["1", "2", "3"].as_slice().into();
            let lines = ["1", "2"];
            let writer = ContainerWriter::init(&mut data, &index)?;
            assert!(writer.write_lines(lines.into_iter()).is_err());

            let data = String::from_utf8(data.into_inner())?;

            assert_eq!(
                data,
                r#"["1","2","3"]
1
2"#
            );
        }

        Ok(())
    }

    fn gen_lines() -> Vec<String> {
        let mut rng = rand::thread_rng();

        (0..30).map(|_| rng.gen_range(1..101).to_string()).collect()
    }

    #[test]
    fn test_read_write_container_lines() -> Result<()> {
        let mut data = Cursor::new(Vec::new());

        let lines = gen_lines();

        {
            let iter = lines.iter().map(|value| value.as_str());
            let writer = ContainerWriter::init(&mut data, &(&lines).into())?;
            writer.write_lines(iter)?;

            data.set_position(0);
        }

        {
            let reader = ContainerReader::init(data)?;

            assert_eq!(*reader.get_index(), lines);

            let new_lines = reader.iter_lines().collect::<Result<Vec<_>>>()?;
            assert_eq!(new_lines, lines);
        }

        Ok(())
    }

    #[test]
    fn test_read_write_gz_container_lines() -> Result<()> {
        let mut data = Cursor::new(Vec::new());

        let lines = gen_lines();

        {
            let mut gz_writer = create_gz_writer(data);
            let iter = lines.iter().map(|value| value.as_str());

            let writer = ContainerWriter::init(&mut gz_writer, &(&lines).into())?;
            writer.write_lines(iter)?;

            data = gz_writer.finish()?;

            data.set_position(0);
        }

        {
            let gz_reader = create_gz_reader(&mut data);
            let reader = ContainerReader::init(gz_reader)?;

            assert_eq!(*reader.get_index(), lines);

            let new_lines = reader.iter_lines().collect::<Result<Vec<_>>>()?;
            assert_eq!(new_lines, lines);
        }

        Ok(())
    }
}
