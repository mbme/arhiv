use std::{
    collections::HashSet,
    io::{BufRead, BufReader, Read, Write},
};

use anyhow::{Context, Result, ensure};
use ordermap::{OrderMap, OrderSet};
use serde::{Deserialize, Serialize};

use crate::{TakeExactly, ZipLongest};

pub type ContainerPatch = OrderMap<String, Option<String>>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(transparent, deny_unknown_fields)]
pub struct LinesIndex {
    index: OrderSet<String>,
}

impl LinesIndex {
    pub fn new(index: impl Iterator<Item = String>) -> Self {
        LinesIndex {
            index: index.collect(),
        }
    }

    fn patch(self, patch: &ContainerPatch) -> Self {
        let mut patched = HashSet::new();

        let mut patched_index: OrderSet<String> = self
            .index
            .into_iter()
            .filter_map(|key| {
                if let Some(patched_value) = patch.get(&key) {
                    patched.insert(key.clone());

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

        let new_keys = patch
            .iter()
            .filter(|(key, value)| !patched.contains(*key) && value.is_some())
            .map(|(key, _value)| key.clone());

        patched_index.extend(new_keys);

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

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.index.iter().map(|value| value.as_str())
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
        self.index
            .iter()
            .map(|value| value.as_str())
            .eq(other.iter().map(|value| value.as_str()))
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

pub struct ContainerReader<R: BufRead> {
    index: LinesIndex,
    reader: R,
}

impl<R: Read> ContainerReader<BufReader<R>> {
    pub fn init(reader: R) -> Result<Self> {
        Self::init_buffered(BufReader::new(reader))
    }
}

impl<R: BufRead> ContainerReader<R> {
    pub fn init_buffered(mut reader: R) -> Result<Self> {
        let mut line = String::new();
        reader.read_line(&mut line)?;

        // support reading empty files
        if line.trim().is_empty() {
            line.push_str("[]");
        }

        let index: LinesIndex =
            serde_json::from_str(&line).context("Failed to parse container index")?;

        Ok(Self { index, reader })
    }

    pub fn get_index(&self) -> &LinesIndex {
        &self.index
    }

    pub fn into_lines_iter(self) -> impl Iterator<Item = Result<(String, String)>> {
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

    pub fn patch<W: Write>(
        self,
        mut writer: ContainerWriter<W>,
        mut patch: ContainerPatch,
    ) -> Result<W> {
        let patched_index = self.index.clone().patch(&patch);
        let iter = self.into_lines_iter();

        writer.write_index(&patched_index)?;

        let patched_iter = iter.filter_map(|result| {
            let (key, value) = {
                match result {
                    Ok((key, value)) => (key, value),
                    Err(err) => {
                        return Some(Err(err));
                    }
                }
            };

            // replace existing value
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

        // write new values at the end
        for line in patch.into_values() {
            let line = line.context("Expected new value, got None")?;

            writer.write_line(&line)?;
        }

        writer.finish()
    }
}

pub struct ContainerWriter<W: Write> {
    writer: W,
    index_written: bool,
    expected_lines_count: usize,
    written_lines_count: usize,
}

impl<W: Write> ContainerWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            index_written: false,
            expected_lines_count: 0,
            written_lines_count: 0,
        }
    }

    pub fn write_index(&mut self, index: &LinesIndex) -> Result<()> {
        ensure!(!self.index_written, "index already written");

        // false positive clippy lint
        #[allow(clippy::needless_borrows_for_generic_args)]
        serde_json::to_writer(&mut self.writer, &index)?;

        self.index_written = true;
        self.expected_lines_count = index.len();

        Ok(())
    }

    pub fn write_lines<'a>(mut self, lines: impl Iterator<Item = &'a str>) -> Result<W> {
        for line in lines {
            self.write_line(line)?;
        }

        self.finish()
    }

    pub fn write_line(&mut self, line: &str) -> Result<()> {
        ensure!(self.index_written, "index not written");

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

    pub fn finish(mut self) -> Result<W> {
        ensure!(self.index_written, "index not written");

        ensure!(
            self.written_lines_count == self.expected_lines_count,
            "Expected {} lines, got {}",
            self.expected_lines_count,
            self.written_lines_count,
        );

        self.writer.flush()?;

        Ok(self.writer)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use anyhow::Result;

    use crate::{ContainerWriter, create_gz_reader, create_gz_writer, generate_alphanumeric_lines};

    use super::{ContainerPatch, ContainerReader, LinesIndex};

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
            let raw_data = "";
            let data = Cursor::new(raw_data.as_bytes());
            let reader = ContainerReader::init_buffered(data)?;
            assert!(reader.get_index().is_empty());

            let new_lines = reader.read_all()?;
            assert_eq!(new_lines, Vec::<&str>::new());
        }

        {
            let raw_data = r#"["1","2","3"]
1
2
3"#;
            let data = Cursor::new(raw_data.as_bytes());
            let reader = ContainerReader::init_buffered(data)?;
            assert_eq!(*reader.get_index(), vec!["1", "2", "3"]);

            let new_lines = reader.read_all()?;
            assert_eq!(new_lines, vec!["1", "2", "3"]);
        }

        {
            let raw_data = r#"["1","2","3"]
1
2"#;
            let data = Cursor::new(raw_data.as_bytes());
            let reader = ContainerReader::init_buffered(data)?;

            assert!(reader.read_all().is_err());
        }

        {
            let raw_data = r#"["1","2","3"]
1
2
3
4"#;
            let data = Cursor::new(raw_data.as_bytes());
            let reader = ContainerReader::init_buffered(data)?;

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
            let mut writer = ContainerWriter::new(&mut data);
            writer.write_index(&index)?;
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
            let mut writer = ContainerWriter::new(&mut data);
            writer.write_index(&index)?;
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
            let mut writer = ContainerWriter::new(&mut data);
            writer.write_index(&index)?;
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

    #[test]
    fn test_read_write_container_lines() -> Result<()> {
        let mut data = Cursor::new(Vec::new());

        let lines = generate_alphanumeric_lines(100, 100);

        {
            let iter = lines.iter().map(|value| value.as_str());
            let mut writer = ContainerWriter::new(&mut data);
            writer.write_index(&(&lines).into())?;
            writer.write_lines(iter)?;

            data.set_position(0);
        }

        {
            let reader = ContainerReader::init_buffered(data)?;

            assert_eq!(*reader.get_index(), lines);

            let new_lines = reader.read_all()?;
            assert_eq!(new_lines, lines);
        }

        Ok(())
    }

    #[test]
    fn test_read_write_gz_container_lines() -> Result<()> {
        let mut data = Cursor::new(Vec::new());

        let lines = generate_alphanumeric_lines(30, 100);

        {
            let mut gz_writer = create_gz_writer(data);
            let iter = lines.iter().map(|value| value.as_str());

            let mut writer = ContainerWriter::new(&mut gz_writer);
            writer.write_index(&(&lines).into())?;
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
    fn test_patch_line_index() -> Result<()> {
        let index: LinesIndex = ["1", "2"].as_slice().into();

        let patch: ContainerPatch = [
            ("1".to_string(), None),
            ("4".to_string(), None),
            ("3".to_string(), Some("3".to_string())),
        ]
        .into();

        let index = index.patch(&patch);

        assert_eq!(index, vec!["2", "3"]);

        Ok(())
    }

    #[test]
    fn test_patch_container() -> Result<()> {
        let mut data = Cursor::new(Vec::new());

        {
            let lines = ["3", "2", "1"];
            let index = lines.as_slice().into();
            let mut writer = ContainerWriter::new(&mut data);
            writer.write_index(&index)?;
            writer.write_lines(lines.into_iter())?;
        }

        {
            data.set_position(0);

            let reader = ContainerReader::init_buffered(&mut data)?;

            let patch: ContainerPatch = [
                ("3".to_string(), None),
                ("1".to_string(), Some("3".to_string())),
                ("4".to_string(), Some("4".to_string())),
            ]
            .into();

            let mut patched_data = Cursor::new(Vec::new());
            reader.patch(ContainerWriter::new(&mut patched_data), patch)?;

            patched_data.set_position(0);

            let reader = ContainerReader::init_buffered(&mut patched_data)?;

            let new_lines = reader.read_all()?;
            assert_eq!(new_lines, vec!["2", "3", "4"]);
        }

        Ok(())
    }
}
