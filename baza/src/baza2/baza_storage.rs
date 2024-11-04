use std::{
    collections::{HashMap, HashSet},
    io::{BufRead, BufReader, Read, Write},
};

use anyhow::{bail, ensure, Context, Result};
use rs_utils::{
    confidential1::{Confidential1Key, Confidential1Reader, Confidential1Writer},
    create_gz_reader, create_gz_writer,
    crypto_key::CryptoKey,
    ContainerReader, ContainerWriter, LinesIndex,
};
use serde::{Deserialize, Serialize};

use crate::entities::{Document, Id, LatestRevComputer, Revision};

fn create_confidential1_gz_container_reader(
    reader: impl BufRead + 'static,
    key: &Confidential1Key,
) -> Result<ContainerReader<BufReader<Box<dyn Read>>>> {
    let c1_reader = Confidential1Reader::new(reader, key)?;
    let c1_buf_reader = BufReader::new(c1_reader);

    let gz_reader = create_gz_reader(c1_buf_reader);

    let boxed_reader: Box<dyn Read> = Box::new(gz_reader);

    let container_reader = ContainerReader::init(boxed_reader)?;

    Ok(container_reader)
}

fn create_confidential1_gz_container_writer(
    writer: impl Write,
    key: &Confidential1Key,
) -> Result<ContainerWriter<impl Write>> {
    let c1_writer = Confidential1Writer::new(writer, key)?;
    let gz_writer = create_gz_writer(c1_writer);

    Ok(ContainerWriter::new(gz_writer))
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BazaInfo {
    pub name: String,
    pub storage_version: u8,
    pub data_version: u8,
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

type LinesIter = Box<dyn Iterator<Item = Result<(String, String)>>>;

enum ReaderOrLinesIter {
    LinesIter(LinesIter),
    Reader(ContainerReader<BufReader<Box<dyn Read>>>),
    Undefined,
}

pub struct BazaStorage<'k> {
    pub index: DocumentsIndex,
    key: Confidential1Key<'k>,
    inner: ReaderOrLinesIter,
    info: Option<BazaInfo>,
}

impl<'k> BazaStorage<'k> {
    pub const VERSION: u8 = 1;

    pub fn create(writer: impl Write, key: &'k CryptoKey, info: &BazaInfo) -> Result<()> {
        let c1_key = Confidential1Key::borrow_key(key);
        let mut c1writer = create_confidential1_gz_container_writer(writer, &c1_key)?;

        let index = DocumentsIndex::create();
        c1writer.write_index(&index)?;
        c1writer.write_line(&serde_json::to_string(info)?)?;
        c1writer.finish()?;

        Ok(())
    }

    pub fn read(reader: impl BufRead + 'static, key: &'k CryptoKey) -> Result<Self> {
        let c1_key = Confidential1Key::borrow_key(key);
        let reader = create_confidential1_gz_container_reader(reader, &c1_key)?;

        let index = DocumentsIndex::parse(reader.get_index())?;
        let inner = ReaderOrLinesIter::Reader(reader);

        Ok(BazaStorage {
            index,
            inner,
            key: c1_key,
            info: None,
        })
    }

    fn get_lines_iter(&mut self) -> &mut LinesIter {
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

    pub fn add(self, writer: impl Write, new_documents: Vec<Document>) -> Result<()> {
        ensure!(!new_documents.is_empty(), "documents to add not provided");

        // prepare patch
        let mut patch = HashMap::with_capacity(new_documents.len());
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
        let c1writer = create_confidential1_gz_container_writer(writer, &self.key)?;

        match self.inner {
            ReaderOrLinesIter::Reader(reader) => {
                reader.patch(c1writer, patch)?;
            }
            _ => bail!("Can only patch Reader"),
        };

        Ok(())
    }

    pub fn merge_all(mut storages: Vec<BazaStorage>, writer: impl Write) -> Result<()> {
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
        let mut c1writer = create_confidential1_gz_container_writer(writer, key)?;

        c1writer.write_index(&index)?;

        // write lines
        for (s, mut keys) in keys_per_storage {
            for line in s {
                if keys.is_empty() {
                    break;
                }

                let (key, line) = line?;

                if key == keys[0] {
                    c1writer.write_line(&line)?;
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

        c1writer.finish()?;

        Ok(())
    }
}

impl<'k> Iterator for BazaStorage<'k> {
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
