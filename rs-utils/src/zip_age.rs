use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

use anyhow::{bail, Context, Result};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

type ZipAgeReader = ZipArchive<BufReader<File>>;

type ZipAgeWriter = ZipWriter<BufWriter<File>>;

pub struct ZipAge {
    archive: ZipAgeReader,
    file_path: String,
}

impl ZipAge {
    pub fn open(file_path: &str) -> Result<Self> {
        let file = File::open(file_path)?;

        let archive = ZipArchive::new(BufReader::new(file))?;

        Ok(ZipAge {
            archive,
            file_path: file_path.to_string(),
        })
    }

    pub fn create(file_path: &str, data: &HashMap<&str, &[u8]>) -> Result<()> {
        let mut new_archive = Self::create_writer(file_path)?;

        for (file_name, value) in data {
            ZipAge::write_file(&mut new_archive, file_name, value)?;
        }

        new_archive.finish()?;

        Ok(())
    }

    pub fn update_and_save(
        &mut self,
        file_path: &str,
        mut data: HashMap<&str, Option<&[u8]>>,
    ) -> Result<()> {
        let mut new_archive = Self::create_writer(file_path)?;

        for i in 0..self.archive.len() {
            let file = self.archive.by_index_raw(i)?;
            let file_name = file.name();

            // replace existing file with a new value
            if let Some(new_value) = data.remove(file_name) {
                if let Some(new_value) = new_value {
                    ZipAge::write_file(&mut new_archive, file_name, new_value)?;
                } else {
                    // intent is to remove the file, so let's skip it
                    continue;
                }
            } else {
                // otherwise, copy compressed data to the new archive
                new_archive.raw_copy_file(file)?;
            }
        }

        // add remaining (new) files to the zip archive
        for (file_name, value) in data {
            if let Some(value) = value {
                ZipAge::write_file(&mut new_archive, file_name, value)?;
            } else {
                bail!("No value provided for a new file {file_name}");
            }
        }

        new_archive.finish()?;

        Ok(())
    }

    pub fn get_path(&self) -> &str {
        &self.file_path
    }

    fn create_writer(file_path: &str) -> Result<ZipAgeWriter> {
        let new_file = File::create(file_path)?;
        let buffered_writer = BufWriter::new(new_file);

        Ok(ZipWriter::new(buffered_writer))
    }

    fn write_file(zip: &mut ZipAgeWriter, file_name: &str, data: &[u8]) -> Result<()> {
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        zip.start_file(file_name, options)?;
        zip.write_all(data)?;

        Ok(())
    }

    pub fn read_file(&mut self, path: &str) -> Result<impl Read + '_> {
        let zip = &mut self.archive;

        let index = zip
            .index_for_name(path)
            .with_context(|| format!("Can't find path '{path}' in zip"))?;

        let file = zip.by_index(index)?;

        Ok(file)
    }

    pub fn get_file_bytes(&mut self, path: &str) -> Result<Vec<u8>> {
        let mut file = self.read_file(path)?;

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        Ok(bytes)
    }

    pub fn list_files(&self) -> impl Iterator<Item = &str> {
        let zip = &self.archive;

        (0..zip.len()).map(|index| {
            zip.name_for_index(index)
                .expect("name must exist for valid indices")
        })
    }

    pub fn list_files_in_dir(&self, dir_path: &str) -> Result<Vec<&str>> {
        let all_files = self.list_files();

        let mut files = Vec::new();
        for file_path in all_files {
            let rel_path = if let Ok(rel_path) = Path::new(file_path).strip_prefix(dir_path) {
                rel_path
            } else {
                continue;
            };

            let file_name = if let Some(file_name) = rel_path.components().next() {
                file_name
            } else {
                continue;
            };

            let file_name = file_name
                .as_os_str()
                .to_str()
                .context("failed to convert file name to str")?;

            files.push(file_name);
        }

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use anyhow::Result;

    use crate::{TempFile, ZipAge};

    #[test]
    fn test_age_zip_create_and_read() -> Result<()> {
        let temp1 = TempFile::new();

        ZipAge::create(
            &temp1.path,
            &HashMap::from([
                ("/test", "test".as_bytes()),
                ("/dir/test1", "test1".as_bytes()),
            ]),
        )?;

        let mut zip = ZipAge::open(&temp1.path)?;

        assert_eq!(
            zip.list_files().collect::<HashSet<_>>(),
            HashSet::from(["/test", "/dir/test1"]),
        );

        assert_eq!(zip.get_file_bytes("/test")?, b"test");
        assert_eq!(zip.get_file_bytes("/dir/test1")?, b"test1");

        Ok(())
    }

    #[test]
    fn test_age_zip_list_files_in_dir() -> Result<()> {
        let temp1 = TempFile::new();

        ZipAge::create(
            &temp1.path,
            &HashMap::from([
                ("/test", "test".as_bytes()),
                ("/dir", "test".as_bytes()),
                ("/dir/test1", "test1".as_bytes()),
            ]),
        )?;

        let zip = ZipAge::open(&temp1.path)?;

        let mut files_in_dir = zip.list_files_in_dir("/")?;
        files_in_dir.sort();

        let mut expected = vec!["test", "dir", "dir"];
        expected.sort();

        assert_eq!(files_in_dir, expected);

        Ok(())
    }

    #[test]
    fn test_age_zip_update_and_save() -> Result<()> {
        let temp1 = TempFile::new();
        let temp2 = TempFile::new();

        ZipAge::create(
            &temp1.path,
            &HashMap::from([
                ("/test", "test".as_bytes()),
                ("/dir", "test".as_bytes()),
                ("/dir/test1", "test1".as_bytes()),
            ]),
        )?;

        {
            let mut zip = ZipAge::open(&temp1.path)?;
            zip.update_and_save(
                &temp2.path,
                HashMap::from([
                    ("/dir", None), //
                    ("/dir/test1", Some("test2".as_bytes())),
                ]),
            )?;
        }

        {
            let mut zip = ZipAge::open(&temp2.path)?;

            assert_eq!(
                zip.list_files().collect::<HashSet<_>>(),
                HashSet::from(["/test", "/dir/test1"]),
            );

            assert_eq!(zip.get_file_bytes("/test")?, b"test");
            assert_eq!(zip.get_file_bytes("/dir/test1")?, b"test2");
        }

        Ok(())
    }
}
