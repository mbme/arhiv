use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};

use anyhow::{anyhow, ensure, Context, Result};

type LineIndex = Vec<String>;

pub fn create_file_reader(file_path: &str) -> Result<BufReader<File>> {
    let file = File::open(file_path)?;

    let data_reader = BufReader::new(file);

    Ok(data_reader)
}

pub fn create_file_writer(file_path: &str) -> Result<BufWriter<File>> {
    let new_file = File::create(file_path)?;
    let data_writer = BufWriter::new(new_file);

    Ok(data_writer)
}

struct TakeExactly<I> {
    iter: I,
    expected: usize,
    count: usize,
}

impl<I> TakeExactly<I> {
    fn new(iter: I, expected: usize) -> Self {
        TakeExactly {
            iter,
            expected,
            count: 0,
        }
    }
}

impl<I, V> Iterator for TakeExactly<I>
where
    I: Iterator<Item = Result<V>>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let next_value = self.iter.next();

        if let Some(next_value) = next_value {
            self.count += 1;

            if self.count > self.expected {
                return Some(Err(anyhow!(
                    "Expected {} items but got {}",
                    self.expected,
                    self.count
                )));
            }

            Some(next_value)
        } else {
            if self.count < self.expected {
                return Some(Err(anyhow!(
                    "Expected {} items but got {}",
                    self.expected,
                    self.count
                )));
            }

            None
        }
    }
}

pub fn read_container_lines(
    mut reader: impl BufRead,
) -> Result<(LineIndex, impl Iterator<Item = Result<(usize, String)>>)> {
    let mut line = String::new();
    reader.read_line(&mut line)?;

    let index: LineIndex =
        serde_json::from_str(&line).context("Failed to parse container index")?;

    let lines = reader.lines().enumerate().map(|(i, result)| {
        let line = result.with_context(|| format!("Failed to read line {i}"))?;

        Ok((i, line))
    });

    let lines = TakeExactly::new(lines, index.len());

    Ok((index, lines))
}

pub fn write_container_lines<'a>(
    mut writer: impl Write,
    index: &LineIndex,
    lines: impl Iterator<Item = &'a str>,
) -> Result<()> {
    serde_json::to_writer(&mut writer, &index)?;

    let mut lines_count = 0;
    for (i, line) in lines.enumerate() {
        ensure!(i < index.len(), "Expected only {} lines", index.len());

        lines_count += 1;

        writer.write_all(b"\n")?;
        writer.write_all(line.as_bytes())?;
    }

    writer.flush()?;

    ensure!(
        lines_count == index.len(),
        "Expected {} lines, got {lines_count}",
        index.len()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use anyhow::Result;
    use rand::Rng;

    use super::{read_container_lines, write_container_lines};

    fn collect_container_lines(
        iter: impl Iterator<Item = Result<(usize, String)>>,
    ) -> Result<Vec<String>> {
        iter.map(|result| {
            let (_, value) = result?;

            Ok(value)
        })
        .collect::<Result<Vec<_>>>()
    }

    #[test]
    fn test_read_container_lines() -> Result<()> {
        {
            let raw_data = r#"["1","2","3"]
1
2
3"#;
            let data = Cursor::new(raw_data.as_bytes());
            let (index, iter) = read_container_lines(data)?;

            assert_eq!(index, vec!["1", "2", "3"]);

            let new_lines = collect_container_lines(iter)?;
            assert_eq!(new_lines, vec!["1", "2", "3"]);
        }

        {
            let raw_data = r#"["1","2","3"]
1
2"#;
            let data = Cursor::new(raw_data.as_bytes());
            let (_index, iter) = read_container_lines(data)?;

            assert!(collect_container_lines(iter).is_err());
        }

        {
            let raw_data = r#"["1","2","3"]
1
2
3
4"#;
            let data = Cursor::new(raw_data.as_bytes());
            let (_index, iter) = read_container_lines(data)?;

            assert!(collect_container_lines(iter).is_err());
        }

        Ok(())
    }

    #[test]
    fn test_write_container_lines() -> Result<()> {
        {
            let mut data = Cursor::new(Vec::new());

            let lines = ["3", "2", "1"];
            let index = lines.iter().map(ToString::to_string).collect();
            write_container_lines(&mut data, &index, lines.into_iter())?;

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

            let index = ["1", "2"].iter().map(ToString::to_string).collect();
            let lines = ["1", "2", "3"];
            assert!(write_container_lines(&mut data, &index, lines.into_iter()).is_err());

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

            let index = ["1", "2", "3"].iter().map(ToString::to_string).collect();
            let lines = ["1", "2"];
            assert!(write_container_lines(&mut data, &index, lines.into_iter()).is_err());

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

        let mut rng = rand::thread_rng();

        let lines: Vec<_> = (0..30).map(|_| rng.gen_range(1..101).to_string()).collect();

        {
            let iter = lines.iter().map(|value| value.as_str());
            write_container_lines(&mut data, &lines, iter)?;

            data.set_position(0);
        }

        {
            let (index, iter) = read_container_lines(&mut data)?;

            assert_eq!(index, lines);

            let new_lines = collect_container_lines(iter)?;
            assert_eq!(new_lines, lines);
        }

        Ok(())
    }
}
