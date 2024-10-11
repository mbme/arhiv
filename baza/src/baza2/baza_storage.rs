use std::{
    collections::{HashMap, HashSet},
    io::{BufReader, Read, Write},
};

use anyhow::{bail, ensure, Context, Result};
use rs_utils::{
    confidential1::{Confidential1Key, Confidential1Reader, Confidential1Writer},
    create_file_reader, create_file_writer, create_gz_reader, create_gz_writer,
    crypto_key::CryptoKey,
    ContainerReader, ContainerWriter, FsTransaction, LinesIndex, Patch,
};
use serde::{Deserialize, Serialize};

use crate::{
    entities::{BLOBId, Document, Id, InstanceId, LatestRevComputer, Revision},
    get_local_blob_ids,
    path_manager::PathManager,
};

fn create_confidential1_gz_container_reader(
    file: &str,
    key: &Confidential1Key,
) -> Result<ContainerReader<BufReader<Box<dyn Read>>>> {
    let reader = create_file_reader(file)?;

    let c1_reader = Confidential1Reader::new(reader, key)?;
    let c1_buf_reader = BufReader::new(c1_reader);

    let gz_reader = create_gz_reader(c1_buf_reader);

    let boxed_reader: Box<dyn Read> = Box::new(gz_reader);

    let container_reader = ContainerReader::init(boxed_reader)?;

    Ok(container_reader)
}

fn create_confidential1_gz_container_writer(
    file: &str,
    key: &Confidential1Key,
) -> Result<ContainerWriter<impl Write>> {
    let writer = create_file_writer(file)?;
    let c1_writer = Confidential1Writer::new(writer, key)?;
    let gz_writer = create_gz_writer(c1_writer);

    Ok(ContainerWriter::new(gz_writer))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BazaInfo {
    pub name: String,
    pub baza_version: u8,
    pub data_version: u8,
}

#[derive(Hash, Eq, PartialEq)]
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

impl DocumentsIndex {
    pub fn parse(index: &LinesIndex) -> Result<Self> {
        let documents_index = index
            .iter()
            .skip(1) // skip info file
            .map(BazaDocumentKey::parse)
            .collect::<Result<Vec<_>>>()?;

        Ok(DocumentsIndex(documents_index))
    }

    pub fn serialize(&self) -> LinesIndex {
        let mut index = self.0.iter().map(|key| key.serialize()).collect::<Vec<_>>();

        index.insert(0, "info".to_string());

        LinesIndex::new(index)
    }

    pub fn get_all_revs(&self) -> Vec<&Revision> {
        self.0.iter().map(|key| &key.rev).collect()
    }

    pub fn append_keys(&mut self, more_keys: Vec<BazaDocumentKey>) {
        self.0.extend(more_keys);
    }

    pub fn compute_latest_revision(&self) -> Result<HashSet<&Revision>> {
        let mut latest_rev_computer = LatestRevComputer::new();

        let revs = self.0.iter().map(|key| &key.rev);
        latest_rev_computer.update(revs)?;

        Ok(latest_rev_computer.get())
    }

    pub fn compute_latest_document_revision(&self, id: &Id) -> Result<HashSet<&Revision>> {
        let revs = self
            .0
            .iter()
            .filter(|key| key.id == *id)
            .map(|key| &key.rev)
            .collect::<Vec<_>>();

        ensure!(!revs.is_empty(), "document {id} must have revisions");

        Ok(Revision::get_latest_rev(&revs))
    }
}

type LinesIter = Box<dyn Iterator<Item = Result<(String, String)>>>;

enum ReaderOrLinesIter {
    LinesIter(LinesIter),
    Reader(ContainerReader<BufReader<Box<dyn Read>>>),
    Undefined,
}

pub struct BazaIterator {
    pub index: DocumentsIndex,
    inner: ReaderOrLinesIter,
    info: Option<BazaInfo>,
}

impl BazaIterator {
    pub fn read(db_file: &str, key: &Confidential1Key) -> Result<Self> {
        let reader = create_confidential1_gz_container_reader(db_file, key)?;

        let index = DocumentsIndex::parse(reader.get_index())?;
        let inner = ReaderOrLinesIter::Reader(reader);

        Ok(BazaIterator {
            index,
            inner,
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

    pub fn patch(self, new_db_file: &str, key: &Confidential1Key, patch: Patch) -> Result<()> {
        let c1writer = create_confidential1_gz_container_writer(new_db_file, key)?;

        match self.inner {
            ReaderOrLinesIter::Reader(reader) => reader.patch(c1writer, patch),
            _ => bail!("Can only patch Reader"),
        }
    }
}

impl Iterator for BazaIterator {
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

pub struct BazaStorage {
    key: Confidential1Key,
    path_manager: PathManager,
}

impl BazaStorage {
    pub fn new(path_manager: PathManager, key: CryptoKey) -> Result<Self> {
        let c1key = Confidential1Key::Key(key);

        Ok(BazaStorage {
            key: c1key,
            path_manager,
        })
    }

    pub fn read(&self) -> Result<BazaIterator> {
        BazaIterator::read(&self.path_manager.db2_file, &self.key)
    }

    pub fn list_blobs(&self) -> Result<HashSet<BLOBId>> {
        let blobs_dir = self.path_manager.db2_data_dir.clone();
        let ids = get_local_blob_ids(&blobs_dir)?;

        Ok(ids)
    }

    pub fn add(
        &mut self,
        instance_id: &InstanceId,
        new_documents: Vec<Document>,
        new_blobs: HashMap<BLOBId, String>,
        tx: &mut FsTransaction,
    ) -> Result<()> {
        ensure!(!new_documents.is_empty(), "documents to add not provided");

        // FIXME use read/write locks

        // backup db file
        let old_db_file = tx.move_to_backup(self.path_manager.db2_file.clone())?;

        // open old db file
        let baza_iter = BazaIterator::read(&old_db_file, &self.key)?;

        // calculate new rev
        let revs = baza_iter.index.get_all_revs();
        let new_rev = Revision::compute_next_rev(revs.as_slice(), instance_id);

        // prepare patch
        let mut patch = HashMap::with_capacity(new_documents.len());
        for mut new_document in new_documents {
            new_document.rev = Some(new_rev.clone());

            let key = BazaDocumentKey::new(new_document.id.clone(), new_rev.clone()).serialize();
            ensure!(
                !patch.contains_key(&key),
                "duplicate new document {}",
                new_document.id
            );

            let value = serde_json::to_string(&new_document)?;

            patch.insert(key, Some(value));
        }

        // move blobs
        for (new_blob_id, file_path) in new_blobs {
            tx.move_file(
                file_path,
                self.path_manager.get_db2_blob_path(&new_blob_id),
                true,
            )?;
        }

        // apply patch & write db
        baza_iter.patch(&self.path_manager.db2_file, &self.key, patch)?;

        Ok(())
    }
}
