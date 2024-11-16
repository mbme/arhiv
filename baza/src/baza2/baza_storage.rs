use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs::File,
    io::{BufReader, Read, Write},
};

use anyhow::{anyhow, bail, ensure, Context, Result};

use rs_utils::{
    confidential1::Confidential1Key, create_file_reader, create_file_writer, crypto_key::CryptoKey,
    C1GzReader, C1GzWriter, ContainerPatch, ContainerReader, ContainerWriter, LinesIndex,
};

use crate::entities::{Document, Id, LatestRevComputer, Revision};

use super::BazaInfo;

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct BazaDocumentKey {
    pub id: Id,
    pub rev: Revision,
}

impl BazaDocumentKey {
    pub fn new(id: Id, rev: Revision) -> Self {
        Self { id, rev }
    }

    pub fn for_document(document: &Document) -> Result<Self> {
        Ok(BazaDocumentKey::new(
            document.id.clone(),
            document.rev.clone(),
        ))
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

impl std::fmt::Debug for BazaDocumentKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[BazaDocumentKey {}]", &self.serialize()))
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

    pub fn from_string_keys(keys: impl Iterator<Item = String>) -> LinesIndex {
        let mut index = keys.collect::<Vec<_>>();

        index.insert(0, "info".to_string());

        LinesIndex::new(index)
    }

    pub fn from_baza_document_keys<'k>(
        items: impl Iterator<Item = &'k BazaDocumentKey>,
    ) -> LinesIndex {
        Self::from_string_keys(items.map(|key| key.serialize()))
    }

    pub fn append_keys(&mut self, more_keys: Vec<BazaDocumentKey>) {
        self.0.extend(more_keys);
    }

    pub fn as_index_map(&self) -> DocumentsIndexMap {
        let mut map: DocumentsIndexMap = HashMap::new();

        // insert all ids & revs into the map
        for key in &self.0 {
            let entry = map.entry(&key.id).or_default();

            entry.insert(&key.rev);
        }

        // calculate max rev per document
        for revs in &mut map.values_mut() {
            let mut latest_rev_computer = LatestRevComputer::new();

            latest_rev_computer.update(revs.iter().copied());

            let mut latest_rev = latest_rev_computer.get();

            std::mem::swap(revs, &mut latest_rev);
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

impl<'i, R: Read + 'i> std::fmt::Debug for BazaStorage<'i, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("BazaStorage: {:?}", &self.index.0))
    }
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
        let patch = create_container_patch(new_documents)?;

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

impl<'i> BazaStorage<'i, BufReader<File>> {
    pub fn read_file(file: &str, key: &'i CryptoKey) -> Result<Self> {
        let storage_reader = create_file_reader(file)?;

        BazaStorage::read(storage_reader, key)
    }

    pub fn add_and_save_to_file(self, file: &str, new_documents: &[Document]) -> Result<()> {
        let mut storage_writer = create_file_writer(file)?;

        self.add(&mut storage_writer, new_documents)?;

        storage_writer.flush()?;

        Ok(())
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

fn create_container_patch(documents: &[Document]) -> Result<ContainerPatch> {
    let mut patch = ContainerPatch::with_capacity(documents.len());
    for new_document in documents {
        let key = BazaDocumentKey::for_document(new_document)?.serialize();
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
    key: &CryptoKey,
    info: &BazaInfo,
    new_documents: &[Document],
) -> Result<()> {
    let c1_key = Confidential1Key::borrow_key(key);
    let c1writer = C1GzWriter::create(writer, &c1_key)?;
    let container_writer = ContainerWriter::new(c1writer);

    let mut patch = create_container_patch(new_documents)?;

    let info = serde_json::to_string(info)?;
    patch.insert_before(0, "info".to_string(), Some(info));

    let c1writer = container_writer.write(patch)?;
    c1writer.finish()?;

    Ok(())
}

pub fn create_empty_storage_file(file: &str, key: &CryptoKey, info: &BazaInfo) -> Result<()> {
    let mut storage_writer = create_file_writer(file)?;
    create_storage(&mut storage_writer, key, info, &[])?;

    storage_writer.flush()?;

    Ok(())
}

#[cfg(test)]
pub fn create_test_storage<'k>(
    key: &'k CryptoKey,
    new_documents: &[Document],
) -> BazaStorage<'k, impl Read> {
    use std::io::Cursor;

    let info = BazaInfo::new_test_info();

    let mut data = Cursor::new(Vec::<u8>::new());
    create_storage(&mut data, key, &info, new_documents).unwrap();
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
            let mut ordered_keys = VecDeque::with_capacity(keys_set.len());

            for index_key in &s.index.0 {
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
    let index = DocumentsIndex::from_baza_document_keys(index_keys);

    let key = &keys_per_storage[0].0.key;
    let c1writer = C1GzWriter::create(writer, key)?;
    let mut container_writer = ContainerWriter::new(c1writer);

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

    let c1writer = container_writer.finish()?;
    c1writer.finish()?;

    Ok(())
}

pub fn merge_storages_to_file(
    info: &BazaInfo,
    storages: Vec<BazaStorage<impl Read>>,
    file: &str,
) -> Result<()> {
    let mut storage_writer = create_file_writer(file)?;

    merge_storages(info, storages, &mut storage_writer)?;

    storage_writer.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use anyhow::Result;

    use rs_utils::crypto_key::CryptoKey;
    use serde_json::json;

    use crate::{baza2::baza_storage::create_test_storage, tests::new_document};

    use super::{create_storage, merge_storages, BazaInfo, BazaStorage};

    #[test]
    fn test_storage() -> Result<()> {
        let mut data = Cursor::new(Vec::<u8>::new());

        // create
        let key = CryptoKey::new_random_key();
        let info = BazaInfo::new_test_info();
        let mut docs1 = vec![new_document(json!({ "test": "a" })).with_rev(json!({ "1": 1 }))];
        create_storage(&mut data, &key, &info, &docs1)?;

        data.set_position(0);

        // read
        {
            let mut storage = BazaStorage::read(&mut data, &key)?;
            assert_eq!(storage.index.0.len(), 1);
            assert_eq!(storage.get_info()?, &info);

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
            let storage = BazaStorage::read(&mut data, &key)?;
            storage.add(&mut data1, &docs2)?;
        }

        data1.set_position(0);

        // read
        {
            docs1.extend(docs2);
            let mut storage = BazaStorage::read(&mut data1, &key)?;
            assert_eq!(storage.index.0.len(), 3);
            assert_eq!(storage.get_info()?, &info);

            let all_items = storage.get_all()?;
            assert_eq!(all_items, docs1);
        }

        Ok(())
    }

    #[test]
    fn test_merge_storages() {
        let key = CryptoKey::new_random_key();
        let info = BazaInfo::new_test_info();

        let doc_a = new_document(json!({ "test": "a" })).with_rev(json!({ "a": 1 }));
        let doc_b = new_document(json!({ "test": "b" })).with_rev(json!({ "b": 1 }));
        let doc_c = new_document(json!({ "test": "c" })).with_rev(json!({ "c": 3 }));
        let doc_d = new_document(json!({ "test": "d" })).with_rev(json!({ "d": 4 }));

        // create storage1
        let docs1 = vec![doc_a.clone(), doc_b.clone()];
        let storage1 = create_test_storage(&key, &docs1);

        // create storage2
        let docs2 = vec![doc_b.clone(), doc_c.clone()];
        let storage2 = create_test_storage(&key, &docs2);

        // create storage3
        let docs3 = vec![doc_a.clone(), doc_c.clone(), doc_d.clone()];
        let storage3 = create_test_storage(&key, &docs3);

        // merge storages
        let mut result = Cursor::new(Vec::<u8>::new());
        merge_storages(&info, vec![storage1, storage2, storage3], &mut result).unwrap();
        result.set_position(0);

        let mut storage = BazaStorage::read(&mut result, &key).unwrap();
        assert_eq!(storage.index.0.len(), 4);
        assert_eq!(storage.get_info().unwrap(), &info);

        let mut all_docs = [doc_a, doc_b, doc_c, doc_d];
        all_docs.sort_by_cached_key(|item| item.id.to_string());

        let mut all_items = storage.get_all().unwrap();
        all_items.sort_by_cached_key(|item| item.id.to_string());

        assert_eq!(all_items, all_docs);
    }
}
