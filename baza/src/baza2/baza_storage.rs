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

use crate::entities::{Document, Id, InstanceId, LatestRevComputer, Revision};

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

pub struct BazaStorage {
    pub index: DocumentsIndex,
    key: Confidential1Key,
    inner: ReaderOrLinesIter,
    info: Option<BazaInfo>,
}

impl BazaStorage {
    pub fn read(reader: impl BufRead + 'static, key: CryptoKey) -> Result<Self> {
        let c1_key = Confidential1Key::Key(key);
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

    pub fn add(
        self,
        instance_id: &InstanceId,
        writer: impl Write,
        new_documents: Vec<Document>,
    ) -> Result<()> {
        ensure!(!new_documents.is_empty(), "documents to add not provided");

        // calculate new rev
        let revs = self.index.get_all_revs();
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
}

impl Iterator for BazaStorage {
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
