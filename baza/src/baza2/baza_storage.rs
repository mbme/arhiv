use std::{
    collections::{HashMap, HashSet},
    io::{BufReader, Read, Write},
};

use anyhow::{anyhow, bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};

use rs_utils::{
    confidential1::Confidential1Key, crypto_key::CryptoKey, C1GzReader, C1GzWriter, ContainerPatch,
    ContainerReader, ContainerWriter, LinesIndex,
};

use crate::entities::{Document, Id, LatestRevComputer, Revision};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BazaInfo {
    pub name: String,
    pub storage_version: u8,
    pub data_version: u8,
}

impl BazaInfo {
    #[cfg(test)]
    pub fn new_test_info() -> Self {
        Self {
            data_version: 1,
            name: "test".to_string(),
            storage_version: STORAGE_VERSION,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct BazaDocumentKey {
    pub id: Id,
    pub rev: Revision,
}

impl BazaDocumentKey {
    pub fn new(id: Id, rev: Revision) -> Self {
        Self { id, rev }
    }

    pub fn parse(value: &str) -> Result<Self> {
        let (id_raw, rev_raw) = value.split_once(' ').context("Failed to split value")?;

        let id = Id::from(id_raw);
        let rev = Revision::from_file_name(rev_raw)?;

        Ok(Self { id, rev })
    }

    pub fn serialize(&self) -> String {
        format!("{} {}", self.id, self.rev.to_file_name())
    }
}

pub struct DocumentsIndex(Vec<BazaDocumentKey>);

pub type DocumentsIndexMap<'i> = HashMap<&'i Id, HashSet<&'i Revision>>;

impl DocumentsIndex {
    pub fn parse(index: &LinesIndex) -> Result<Self> {
        let documents_index = index
            .iter()
            .skip(1) // skip info file
            .map(BazaDocumentKey::parse)
            .collect::<Result<Vec<_>>>()?;

        Ok(DocumentsIndex(documents_index))
    }

    pub fn create() -> LinesIndex {
        Self::serialize([].iter())
    }

    pub fn serialize<'k>(items: impl Iterator<Item = &'k BazaDocumentKey>) -> LinesIndex {
        let mut index = items.map(|key| key.serialize()).collect::<Vec<_>>();

        index.insert(0, "info".to_string());

        LinesIndex::new(index)
    }

    pub fn append_keys(&mut self, more_keys: Vec<BazaDocumentKey>) {
        self.0.extend(more_keys);
    }

    pub fn as_index_map(&self) -> DocumentsIndexMap {
        let mut map: DocumentsIndexMap = HashMap::new();

        for key in &self.0 {
            let entry = map.entry(&key.id).or_default();

            entry.insert(&key.rev);
        }

        // calculate max rev per document
        for value in &mut map.values_mut() {
            let mut latest_rev_computer = LatestRevComputer::new();

            latest_rev_computer.update(value.iter().copied());

            let mut latest_rev = latest_rev_computer.get();

            std::mem::swap(value, &mut latest_rev);
        }

        map
    }
}

type LinesIter<'i> = Box<dyn Iterator<Item = Result<(String, String)>> + 'i>;

#[allow(clippy::large_enum_variant)]
enum ReaderOrLinesIter<'i, R: Read> {
    LinesIter(LinesIter<'i>),
    Reader(ContainerReader<BufReader<C1GzReader<R>>>),
    Undefined,
}

pub const STORAGE_VERSION: u8 = 1;

pub struct BazaStorage<'i, R: Read + 'i> {
    pub index: DocumentsIndex,
    key: Confidential1Key<'i>,
    inner: ReaderOrLinesIter<'i, R>,
    info: Option<BazaInfo>,
}

impl<'i, R: Read + 'i> BazaStorage<'i, R> {
    pub fn read(reader: R, key: &'i CryptoKey) -> Result<Self> {
        let c1_key = Confidential1Key::borrow_key(key);

        let c1gz_reader = C1GzReader::create(reader, &c1_key)?;
        let reader = ContainerReader::init(c1gz_reader)?;

        let index = DocumentsIndex::parse(reader.get_index())?;
        let inner = ReaderOrLinesIter::Reader(reader);

        Ok(BazaStorage {
            index,
            inner,
            key: c1_key,
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

    pub fn get_info(&mut self) -> Result<&BazaInfo> {
        if let Some(ref info) = self.info {
            return Ok(info);
        }

        let lines = self.get_lines_iter();

        let (_, info) = lines.next().context("failed to read info")??;
        let info = serde_json::from_str(&info)?;
        self.info = Some(info);

        Ok(self.info.as_ref().expect("info is available"))
    }

    pub fn add(self, writer: impl Write, new_documents: &[Document]) -> Result<()> {
        ensure!(!new_documents.is_empty(), "documents to add not provided");

        // prepare patch
        let mut patch = ContainerPatch::with_capacity(new_documents.len());
        for new_document in new_documents {
            let key =
                BazaDocumentKey::new(new_document.id.clone(), new_document.get_rev()?.clone())
                    .serialize();
            ensure!(
                !patch.contains_key(&key),
                "duplicate new document {}",
                new_document.id
            );

            let value = serde_json::to_string(&new_document)?;

            patch.insert(key, Some(value));
        }

        // apply patch & write db
        let c1writer = C1GzWriter::create(writer, &self.key)?;
        let container_writer = ContainerWriter::new(c1writer);

        match self.inner {
            ReaderOrLinesIter::Reader(reader) => {
                let c1writer = reader.patch(container_writer, patch)?;
                c1writer.finish()?;
            }
            _ => bail!("Can only patch Reader"),
        };

        Ok(())
    }

    pub fn next_parsed(&mut self) -> Option<Result<(BazaDocumentKey, Document)>> {
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
        let mut all_items = Vec::with_capacity(self.index.0.len());

        while let Some(result) = self.next_parsed() {
            let (_key, document) = result?;

            all_items.push(document);
        }

        Ok(all_items)
    }
}

impl<'i, R: Read + 'i> Iterator for BazaStorage<'i, R> {
    type Item = Result<(BazaDocumentKey, String)>;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.get_lines_iter().next()?;

        match line {
            Ok((ref key_raw, line)) => {
                let key = BazaDocumentKey::parse(key_raw).expect("must be valid document key");

                Some(Ok((key, line)))
            }
            Err(err) => Some(Err(err)),
        }
    }
}

pub fn create_storage(writer: impl Write, key: &CryptoKey, info: &BazaInfo) -> Result<()> {
    let c1_key = Confidential1Key::borrow_key(key);
    let c1writer = C1GzWriter::create(writer, &c1_key)?;
    let mut container_writer = ContainerWriter::new(c1writer);

    let index = DocumentsIndex::create();
    container_writer.write_index(&index)?;
    container_writer.write_line(&serde_json::to_string(info)?)?;
    let c1writer = container_writer.finish()?;
    c1writer.finish()?;

    Ok(())
}

pub fn merge_storages(mut storages: Vec<BazaStorage<impl Read>>, writer: impl Write) -> Result<()> {
    ensure!(!storages.is_empty(), "storages must not be empty");

    let same_info = storages
        .iter_mut()
        .map(|s| s.get_info())
        .collect::<Result<Vec<_>>>()?
        .windows(2)
        .all(|w| w[0] == w[1]);
    ensure!(same_info, "all storages must have same info");

    let mut keys_per_storage = storages
        .into_iter()
        .map(|s| {
            let keys = HashSet::<BazaDocumentKey>::from_iter(s.index.0.iter().cloned());

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
            let mut ordered_keys = Vec::with_capacity(keys_set.len());

            for index_key in &s.index.0 {
                if let Some(key) = keys_set.take(index_key) {
                    ordered_keys.push(key);
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
    let index = DocumentsIndex::serialize(index_keys);

    let key = &keys_per_storage[0].0.key;
    let c1writer = C1GzWriter::create(writer, key)?;
    let mut container_writer = ContainerWriter::new(c1writer);

    container_writer.write_index(&index)?;

    // write lines
    for (s, mut keys) in keys_per_storage {
        for line in s {
            if keys.is_empty() {
                break;
            }

            let (key, line) = line?;

            if key == keys[0] {
                container_writer.write_line(&line)?;
                keys.pop();
            }
        }

        if !keys.is_empty() {
            bail!(
                "{} keys left after reading all lines from the storage",
                keys.len()
            );
        }
    }

    let c1writer = container_writer.finish()?;
    c1writer.finish()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use anyhow::Result;

    use rs_utils::crypto_key::CryptoKey;
    use serde_json::json;

    use crate::tests::new_document;

    use super::{create_storage, BazaInfo, BazaStorage};

    #[test]
    fn test_storage() -> Result<()> {
        let mut data = Cursor::new(Vec::<u8>::new());

        // create
        let key = CryptoKey::new_random_key();
        let info = BazaInfo::new_test_info();
        create_storage(&mut data, &key, &info)?;

        data.set_position(0);

        // read
        {
            let mut storage = BazaStorage::read(&mut data, &key)?;
            assert_eq!(storage.index.0.len(), 0);
            assert_eq!(storage.get_info()?, &info);
            assert!(storage.next().is_none());
        }

        data.set_position(0);

        // add
        let mut data1 = Cursor::new(Vec::<u8>::new());
        let docs = vec![
            new_document(json!({ "test": "a" })).with_rev(json!({ "1": 1 })),
            new_document(json!({ "test": "b" })).with_rev(json!({ "1": 2 })),
        ];
        {
            let storage = BazaStorage::read(&mut data, &key)?;
            storage.add(&mut data1, &docs)?;
        }

        data1.set_position(0);

        // read
        {
            let mut storage = BazaStorage::read(&mut data1, &key)?;
            assert_eq!(storage.index.0.len(), 2);
            assert_eq!(storage.get_info()?, &info);

            let all_items = storage.get_all()?;
            assert_eq!(all_items, docs);
        }

        Ok(())
    }
}
