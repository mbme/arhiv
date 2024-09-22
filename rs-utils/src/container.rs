use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
};

use anyhow::{ensure, Context, Result};
use flate2::{bufread::GzDecoder, write::GzEncoder, Compression};
use serde::{Deserialize, Serialize};

use crate::{
    confidential1::{create_confidential1_reader, create_confidential1_writer, Confidential1Key},
    create_file_reader, create_file_writer, TakeExactly, ZipLongest,
};

pub type Patch = HashMap<String, Option<String>>;

pub type LineKey = String;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(transparent, deny_unknown_fields)]
pub struct LinesIndex {
    index: Vec<LineKey>,
}

impl LinesIndex {
    fn patch(self, patch: &Patch) -> Self {
        let patched_index = self
            .index
            .into_iter()
            .filter_map(|key| {
                if let Some(patched_value) = patch.get(&key) {
                    if patched_value.is_some() {
                        Some(key)
                    } else {
                        None
                    }
                } else {
                    Some(key)
                }
            })
            .collect();

        Self {
            index: patched_index,
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.index.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a> From<&[&'a str]> for LinesIndex {
    fn from(value: &[&'a str]) -> Self {
        let index = value.iter().map(|line| (*line).into()).collect();

        Self { index }
    }
}

impl From<&[String]> for LinesIndex {
    fn from(value: &[String]) -> Self {
        let index = value.iter().map(|value| value.as_str().into()).collect();

        Self { index }
    }
}

impl From<&Vec<String>> for LinesIndex {
    fn from(value: &Vec<String>) -> Self {
        value.as_slice().into()
    }
}

impl PartialEq<Vec<String>> for LinesIndex {
    fn eq(&self, other: &Vec<String>) -> bool {
        self.index == *other
    }
}

impl PartialEq<Vec<&str>> for LinesIndex {
    fn eq(&self, other: &Vec<&str>) -> bool {
        self.index
            .iter()
            .map(|value| value.as_str())
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

pub struct ContainerReader<R: BufRead> {
    index: LinesIndex,
    reader: R,
}

impl<R: BufRead> ContainerReader<R> {
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

    pub fn into_lines_iter(self) -> impl Iterator<Item = Result<(LineKey, String)>> {
        let index_len = self.index.len();

        let index_iter = self.index.index.into_iter();
        let lines_iter = self
            .reader
            .lines()
            .enumerate()
            .map(|(i, result)| result.with_context(|| format!("Failed to read line {i}")));

        let zipped = ZipLongest::new(index_iter, lines_iter).map(|(key, line)| {
            let key = key.context("no key for line")?;
            let line = line.context("no line for key")??;

            Ok((key, line))
        });

        TakeExactly::new(zipped, index_len)
    }

    pub fn read_all(self) -> Result<Vec<String>> {
        self.into_lines_iter()
            .map(|result| result.map(|(_key, value)| value))
            .collect()
    }

    pub fn patch<W: Write>(self, writer: W, mut patch: Patch) -> Result<()> {
        let patched_index = self.index.clone().patch(&patch);
        let iter = self.into_lines_iter();

        let mut writer = ContainerWriter::init(writer, &patched_index)?;

        let patched_iter = iter.filter_map(|result| {
            let (key, value) = {
                match result {
                    Ok((key, value)) => (key, value),
                    Err(err) => {
                        return Some(Err(err));
                    }
                }
            };

            if let Some(patched_value) = patch.remove(&key) {
                patched_value.map(Ok)
            } else {
                Some(Ok(value))
            }
        });

        for line in patched_iter {
            let line = line?;

            writer.write_line(&line)?;
        }

        writer.finish()
    }
}

pub struct ContainerWriter<W: Write> {
    writer: W,
    expected_lines_count: usize,
    written_lines_count: usize,
}

impl<W: Write> ContainerWriter<W> {
    pub fn init(mut writer: W, index: &LinesIndex) -> Result<Self> {
        // false positive clippy lint
        #[allow(clippy::needless_borrows_for_generic_args)]
        serde_json::to_writer(&mut writer, &index)?;

        Ok(Self {
            writer,
            expected_lines_count: index.len(),
            written_lines_count: 0,
        })
    }

    pub fn write_lines<'a>(mut self, lines: impl Iterator<Item = &'a str>) -> Result<()> {
        for line in lines {
            self.write_line(line)?;
        }

        self.finish()
    }

    pub fn write_line(&mut self, line: &str) -> Result<()> {
        let new_lines_count = self.written_lines_count + 1;

        ensure!(
            new_lines_count <= self.expected_lines_count,
            "Expected only {} lines",
            self.expected_lines_count
        );

        self.writer.write_all(b"\n")?;
        self.writer.write_all(line.as_bytes())?;

        self.written_lines_count = new_lines_count;

        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        self.writer.flush()?;

        ensure!(
            self.written_lines_count == self.expected_lines_count,
            "Expected {} lines, got {}",
            self.expected_lines_count,
            self.written_lines_count,
        );

        Ok(())
    }
}

pub fn create_confidential1_gz_container_reader(
    file: &str,
    key: &Confidential1Key,
) -> Result<ContainerReader<impl BufRead>> {
    let reader = create_file_reader(file)?;
    let confidential1_reader = create_confidential1_reader(reader, key)?;
    let gz_reader = create_gz_reader(confidential1_reader);

    ContainerReader::init(gz_reader)
}

pub fn create_confidential1_gz_container_writer(
    file: &str,
    key: &Confidential1Key,
    index: &LinesIndex,
) -> Result<ContainerWriter<impl Write>> {
    let writer = create_file_writer(file)?;
    let confidential1_writer = create_confidential1_writer(writer, key)?;
    let gz_writer = create_gz_writer(confidential1_writer);

    ContainerWriter::init(gz_writer, index)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use anyhow::Result;
    use rand::Rng;

    use crate::{create_gz_reader, ContainerWriter};

    use super::{create_gz_writer, ContainerReader, LinesIndex, Patch};

    #[test]
    fn test_line_index_serialization() -> Result<()> {
        let raw_index = r#"["1","2","3"]"#;
        let index: LinesIndex = serde_json::from_str(raw_index)?;

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

            let new_lines = reader.read_all()?;
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

            let new_lines = reader.read_all()?;
            assert_eq!(new_lines, lines);
        }

        Ok(())
    }

    #[test]
    fn test_patch_container() -> Result<()> {
        let mut data = Cursor::new(Vec::new());

        {
            let lines = ["3", "2", "1"];
            let index = lines.as_slice().into();
            let writer = ContainerWriter::init(&mut data, &index)?;
            writer.write_lines(lines.into_iter())?;
        }

        {
            data.set_position(0);

            let reader = ContainerReader::init(&mut data)?;

            let patch: Patch = [
                ("3".to_string(), None),
                ("1".to_string(), Some("3".to_string())),
            ]
            .into();

            let mut patched_data = Cursor::new(Vec::new());
            reader.patch(&mut patched_data, patch)?;

            patched_data.set_position(0);

            let reader = ContainerReader::init(&mut patched_data)?;

            let new_lines = reader.read_all()?;
            assert_eq!(new_lines, vec!["2", "3"]);
        }

        Ok(())
    }
}
