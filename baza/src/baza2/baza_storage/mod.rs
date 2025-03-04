mod documents_index;

use std::{
    collections::{HashSet, VecDeque},
    fmt,
    fs::File,
    io::{BufReader, Read, Write},
    time::Instant,
};

use anyhow::{anyhow, bail, ensure, Context, Result};

use rs_utils::{
    age::AgeKey, create_file_reader, create_file_writer, log, AgeGzReader, AgeGzWriter,
    ContainerPatch, ContainerReader, ContainerWriter,
};

use crate::entities::{Document, DocumentKey};

use super::BazaInfo;

pub use documents_index::DocumentsIndex;

type LinesIter<'i> = Box<dyn Iterator<Item = Result<(String, String)>> + 'i>;

#[allow(clippy::large_enum_variant)]
enum ReaderOrLinesIter<'i, R: Read> {
    LinesIter(LinesIter<'i>),
    Reader(ContainerReader<BufReader<AgeGzReader<R>>>),
    Undefined,
}

pub const STORAGE_VERSION: u8 = 1;

pub struct BazaStorage<'i, R: Read + 'i> {
    pub index: DocumentsIndex,
    key: AgeKey,
    inner: ReaderOrLinesIter<'i, R>,
    info: Option<BazaInfo>,
}

impl<'i, R: Read + 'i> fmt::Debug for BazaStorage<'i, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("BazaStorage: {:?}", &self.index))
    }
}

impl<'i, R: Read + 'i> BazaStorage<'i, R> {
    pub fn read(reader: R, key: AgeKey) -> Result<Self> {
        let agegz_reader = AgeGzReader::new(reader, key.clone())?;
        let reader = ContainerReader::init(agegz_reader)?;

        let index =
            DocumentsIndex::parse(reader.get_index()).context("Failed to parse DocumentsIndex")?;
        let inner = ReaderOrLinesIter::Reader(reader);

        Ok(BazaStorage {
            index,
            inner,
            key,
            info: None,
        })
    }

    fn get_lines_iter(&mut self) -> &mut LinesIter<'i> {
        if let ReaderOrLinesIter::Reader(_) = self.inner {
            let mut inner = ReaderOrLinesIter::Undefined;
            std::mem::swap(&mut inner, &mut self.inner);

            match inner {
                ReaderOrLinesIter::Reader(reader) => {
                    let iter = Box::new(reader.into_lines_iter());

                    self.inner = ReaderOrLinesIter::LinesIter(iter);
                }
                _ => unreachable!("must be Reader"),
            }
        }

        match &mut self.inner {
            ReaderOrLinesIter::LinesIter(ref mut iter) => iter,
            _ => unreachable!("must be LinesIter"),
        }
    }

    fn read_info_if_necessary(&mut self) -> Result<()> {
        if self.info.is_some() {
            return Ok(());
        }

        let lines = self.get_lines_iter();

        let (_, info) = lines.next().context("failed to read info")??;
        let info = serde_json::from_str(&info)?;
        self.info = Some(info);

        Ok(())
    }

    pub fn get_info(&mut self) -> Result<&BazaInfo> {
        self.read_info_if_necessary()?;

        Ok(self.info.as_ref().expect("info is available"))
    }

    pub fn patch(self, writer: impl Write, patch: ContainerPatch) -> Result<()> {
        ensure!(!patch.is_empty(), "container patch must not be empty");

        // apply patch & write db
        let agegz_writer = AgeGzWriter::new(writer, self.key)?;
        let container_writer = ContainerWriter::new(agegz_writer);

        match self.inner {
            ReaderOrLinesIter::Reader(reader) => {
                let agegz_writer = reader.patch(container_writer, patch)?;
                agegz_writer.finish()?;
            }
            _ => bail!("Can only patch Reader"),
        };

        Ok(())
    }

    pub fn next_parsed(&mut self) -> Option<Result<(DocumentKey, Document)>> {
        let value = self.next()?;

        let new_value = value.and_then(|(key, raw_document)| {
            let document = serde_json::from_str(&raw_document)
                .context(anyhow!("Failed to parse document {}", key.serialize()))?;

            Ok((key, document))
        });

        Some(new_value)
    }

    #[cfg(test)]
    pub fn get_all(mut self) -> Result<Vec<Document>> {
        let mut all_items = Vec::with_capacity(self.index.len());

        while let Some(result) = self.next_parsed() {
            let (_key, document) = result?;

            all_items.push(document);
        }

        Ok(all_items)
    }

    pub fn contains(&self, key: &DocumentKey) -> bool {
        self.index.contains(key)
    }
}

pub type BazaFileStorage<'i> = BazaStorage<'i, BufReader<File>>;

impl BazaFileStorage<'_> {
    pub fn read_file(file: &str, key: AgeKey) -> Result<Self> {
        log::debug!("Reading storage from file {file}");

        let start_time = Instant::now();

        let storage_reader = create_file_reader(file)?;

        let storage = BazaStorage::read(storage_reader, key)?;

        let duration = start_time.elapsed();
        log::info!("Read storage from file in {:?}", duration);

        Ok(storage)
    }

    pub fn patch_and_save_to_file(self, file: &str, patch: ContainerPatch) -> Result<()> {
        log::debug!("Writing storage to file {file}");

        let start_time = Instant::now();

        let mut storage_writer =
            create_file_writer(file, false).context("Failed to create storage file writer")?;

        self.patch(&mut storage_writer, patch)?;

        storage_writer.flush()?;

        let duration = start_time.elapsed();
        log::info!("Wrote storage to file in {:?}", duration);

        Ok(())
    }
}

impl<'i, R: Read + 'i> Iterator for BazaStorage<'i, R> {
    type Item = Result<(DocumentKey, String)>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Err(err) = self.read_info_if_necessary() {
            return Some(Err(err));
        }

        let line = self.get_lines_iter().next()?;

        match line {
            Ok((ref key_raw, line)) => {
                let key = DocumentKey::parse(key_raw).expect("must be valid document key");

                Some(Ok((key, line)))
            }
            Err(err) => Some(Err(err)),
        }
    }
}

pub fn create_container_patch<'d>(
    documents: impl Iterator<Item = &'d Document>,
) -> Result<ContainerPatch> {
    let mut patch = ContainerPatch::new();
    for new_document in documents {
        let key = DocumentKey::for_document(new_document).serialize();
        ensure!(
            !patch.contains_key(&key),
            "duplicate new document {}",
            new_document.id
        );

        let value = serde_json::to_string(&new_document)?;

        patch.insert(key, Some(value));
    }

    Ok(patch)
}

pub fn create_storage(
    writer: impl Write,
    key: AgeKey,
    info: &BazaInfo,
    new_documents: &[Document],
) -> Result<()> {
    let agegz_writer = AgeGzWriter::new(writer, key)?;
    let mut container_writer = ContainerWriter::new(agegz_writer);

    let index =
        DocumentsIndex::from_document_keys(new_documents.iter().map(DocumentKey::for_document));

    container_writer.write_index(&index)?;

    // first line is BazaInfo
    let info = serde_json::to_string(info)?;
    container_writer.write_line(&info)?;

    for document in new_documents {
        let value = serde_json::to_string(document)?;
        container_writer.write_line(&value)?;
    }

    let agegz_writer = container_writer.finish()?;

    agegz_writer.finish()?;

    Ok(())
}

pub fn create_empty_storage_file(file: &str, key: AgeKey, info: &BazaInfo) -> Result<()> {
    let mut storage_writer = create_file_writer(file, false)?;
    create_storage(&mut storage_writer, key, info, &[])?;

    storage_writer.flush()?;

    Ok(())
}

#[cfg(test)]
pub fn create_test_storage<'k>(
    key: AgeKey,
    new_documents: &[Document],
) -> BazaStorage<'k, impl Read> {
    use std::io::Cursor;

    let info = BazaInfo::new_test_info();

    let mut data = Cursor::new(Vec::<u8>::new());
    create_storage(&mut data, key.clone(), &info, new_documents).unwrap();
    data.set_position(0);

    BazaStorage::read(data, key).unwrap()
}

pub fn merge_storages(
    info: &BazaInfo,
    mut storages: Vec<BazaStorage<impl Read>>,
    writer: impl Write,
) -> Result<()> {
    ensure!(!storages.is_empty(), "storages must not be empty");

    let is_same_info = storages
        .iter_mut()
        .map(|s| s.get_info())
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .all(|s_info| s_info == info);
    ensure!(is_same_info, "all storages must have same info");

    let mut keys_per_storage = storages
        .into_iter()
        .map(|s| {
            let keys = HashSet::<DocumentKey>::from_iter(s.index.iter().cloned());

            (s, keys)
        })
        .collect::<Vec<_>>();

    // Set Cover problem: greedy algorithm
    for i in 0..keys_per_storage.len() {
        let (left, right) = keys_per_storage.split_at_mut(i);

        // sort by the number of unique keys, descending
        right.sort_by(|s1, s2| s2.1.len().cmp(&s1.1.len()));

        if let Some((_, l_keys)) = left.last() {
            for (_, r_keys) in right {
                r_keys.retain(|item| !l_keys.contains(item));
            }
        }
    }

    // ordered unique storage keys
    let keys_per_storage = keys_per_storage
        .into_iter()
        .filter(|(_s, keys_set)| !keys_set.is_empty())
        .map(|(s, mut keys_set)| {
            let mut ordered_keys = VecDeque::with_capacity(keys_set.len());

            for index_key in s.index.iter() {
                if let Some(key) = keys_set.take(index_key) {
                    ordered_keys.push_back(key);
                }
            }

            if !keys_set.is_empty() {
                bail!("{} keys left after ordering", keys_set.len());
            }

            Ok((s, ordered_keys))
        })
        .collect::<Result<Vec<_>>>()?;

    // build index
    let index_keys = keys_per_storage.iter().flat_map(|(_s, keys)| keys.iter());
    let index = DocumentsIndex::from_document_keys_refs(index_keys);

    let key = &keys_per_storage[0].0.key;
    let agegz_writer = AgeGzWriter::new(writer, key.clone())?;
    let mut container_writer = ContainerWriter::new(agegz_writer);

    container_writer.write_index(&index)?;
    container_writer.write_line(&serde_json::to_string(&info)?)?;

    // write lines
    for (storage, mut keys) in keys_per_storage {
        for line in storage {
            if let Some(key) = keys.front() {
                let (line_key, line) = line?;

                if key == &line_key {
                    container_writer.write_line(&line)?;
                    keys.pop_front();
                }
            } else {
                break;
            }
        }

        if !keys.is_empty() {
            bail!(
                "{} keys left after reading all lines from the storage",
                keys.len()
            );
        }
    }

    let agegz_writer = container_writer.finish()?;
    agegz_writer.finish()?;

    Ok(())
}

pub fn merge_storages_to_file(
    info: &BazaInfo,
    storages: Vec<BazaStorage<impl Read>>,
    file: &str,
) -> Result<()> {
    log::debug!("Merging {} storages to file {file}", storages.len());

    let start_time = Instant::now();

    let mut storage_writer = create_file_writer(file, false)?;

    merge_storages(info, storages, &mut storage_writer)?;

    storage_writer.flush()?;

    let duration = start_time.elapsed();
    log::info!("Merged storages to file in {:?}", duration);

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use anyhow::Result;

    use rs_utils::age::AgeKey;
    use serde_json::json;

    use crate::{baza2::baza_storage::create_test_storage, tests::new_document};

    use super::{create_container_patch, create_storage, merge_storages, BazaInfo, BazaStorage};

    #[test]
    fn test_storage() -> Result<()> {
        let mut data = Cursor::new(Vec::<u8>::new());

        // create
        let key = AgeKey::generate_age_x25519_key();
        let info = BazaInfo::new_test_info();
        let mut docs1 = vec![new_document(json!({ "test": "a" })).with_rev(json!({ "1": 1 }))];
        create_storage(&mut data, key.clone(), &info, &docs1)?;

        data.set_position(0);

        // read
        {
            let mut storage = BazaStorage::read(&mut data, key.clone())?;
            assert_eq!(storage.index.len(), 1);
            assert_eq!(storage.get_info()?, &info);

            let all_items = storage.get_all()?;
            assert_eq!(all_items, docs1);
        }

        data.set_position(0);

        // read without reading info first
        {
            let storage = BazaStorage::read(&mut data, key.clone())?;
            let all_items = storage.get_all()?;
            assert_eq!(all_items, docs1);
        }

        data.set_position(0);

        // add
        let mut data1 = Cursor::new(Vec::<u8>::new());
        let docs2 = vec![
            new_document(json!({ "test": "b" })).with_rev(json!({ "1": 2 })),
            new_document(json!({ "test": "c" })).with_rev(json!({ "1": 3 })),
        ];
        {
            let storage = BazaStorage::read(&mut data, key.clone())?;
            storage.patch(&mut data1, create_container_patch(docs2.iter())?)?;
        }

        data1.set_position(0);

        // read
        {
            docs1.extend(docs2);
            let mut storage = BazaStorage::read(&mut data1, key.clone())?;
            assert_eq!(storage.index.len(), 3);
            assert_eq!(storage.get_info()?, &info);

            let all_items = storage.get_all()?;
            assert_eq!(all_items, docs1);
        }

        Ok(())
    }

    #[test]
    fn test_merge_storages() {
        let key = AgeKey::generate_age_x25519_key();
        let info = BazaInfo::new_test_info();

        let doc_a = new_document(json!({ "test": "a" })).with_rev(json!({ "a": 1 }));
        let doc_b = new_document(json!({ "test": "b" })).with_rev(json!({ "b": 1 }));
        let doc_c = new_document(json!({ "test": "c" })).with_rev(json!({ "c": 3 }));
        let doc_d = new_document(json!({ "test": "d" })).with_rev(json!({ "d": 4 }));

        // create storage1
        let docs1 = vec![doc_a.clone(), doc_b.clone()];
        let storage1 = create_test_storage(key.clone(), &docs1);

        // create storage2
        let docs2 = vec![doc_b.clone(), doc_c.clone()];
        let storage2 = create_test_storage(key.clone(), &docs2);

        // create storage3
        let docs3 = vec![doc_a.clone(), doc_c.clone(), doc_d.clone()];
        let storage3 = create_test_storage(key.clone(), &docs3);

        // merge storages
        let mut result = Cursor::new(Vec::<u8>::new());
        merge_storages(&info, vec![storage1, storage2, storage3], &mut result).unwrap();
        result.set_position(0);

        let mut storage = BazaStorage::read(&mut result, key.clone()).unwrap();
        assert_eq!(storage.index.len(), 4);
        assert_eq!(storage.get_info().unwrap(), &info);

        let mut all_docs = [doc_a, doc_b, doc_c, doc_d];
        all_docs.sort_by_cached_key(|item| item.id.to_string());

        let mut all_items = storage.get_all().unwrap();
        all_items.sort_by_cached_key(|item| item.id.to_string());

        assert_eq!(all_items, all_docs);
    }
}
