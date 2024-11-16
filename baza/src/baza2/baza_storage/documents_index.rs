use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use anyhow::Result;

use rs_utils::LinesIndex;

use crate::entities::{Id, LatestRevComputer, Revision};

use super::BazaDocumentKey;

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

    pub fn from_document_keys_refs<'k>(
        items: impl Iterator<Item = &'k BazaDocumentKey>,
    ) -> LinesIndex {
        Self::from_string_keys(items.map(|key| key.serialize()))
    }

    pub fn from_document_keys(items: impl Iterator<Item = BazaDocumentKey>) -> LinesIndex {
        Self::from_string_keys(items.map(|key| key.serialize()))
    }

    pub fn len(&self) -> usize {
        self.0.len()
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

    pub fn iter(&self) -> impl Iterator<Item = &BazaDocumentKey> {
        self.0.iter()
    }

    pub fn contains(&self, key: &BazaDocumentKey) -> bool {
        self.0.iter().any(|value| value == key)
    }
}

impl fmt::Debug for DocumentsIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
