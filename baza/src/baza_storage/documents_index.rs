use std::fmt;

use anyhow::{Context, Result, bail};
use ordermap::OrderSet;

use rs_utils::LinesIndex;

use super::DocumentKey;

pub struct DocumentsIndex(OrderSet<DocumentKey>);

impl DocumentsIndex {
    pub fn parse(index: &LinesIndex) -> Result<Self> {
        let documents_index = index
            .iter()
            .skip(1) // skip info file
            .map(DocumentKey::parse)
            .collect::<Result<OrderSet<_>>>()
            .context("Failed to parse DocumentKey")?;

        Ok(DocumentsIndex(documents_index))
    }

    pub fn from_document_keys(keys: Vec<DocumentKey>) -> Result<Self> {
        let mut set = OrderSet::with_capacity(keys.len());

        for key in keys {
            if set.contains(&key) {
                bail!("duplicate document key {}", key.serialize())
            }

            set.insert(key);
        }

        Ok(DocumentsIndex(set))
    }

    pub fn to_lines_index(&self) -> LinesIndex {
        let mut index = self.iter().map(|key| key.serialize()).collect::<Vec<_>>();

        index.insert(0, "info".to_string());

        LinesIndex::new(index.into_iter())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn position_of(&self, key: &DocumentKey) -> Option<usize> {
        self.0.get_index_of(key)
    }

    pub fn key_at(&self, index: usize) -> Option<&DocumentKey> {
        self.0.get_index(index)
    }

    pub fn append_keys(&mut self, more_keys: Vec<DocumentKey>) {
        self.0.extend(more_keys);
    }

    pub fn iter(&self) -> impl Iterator<Item = &DocumentKey> {
        self.0.iter()
    }

    pub fn contains(&self, key: &DocumentKey) -> bool {
        self.0.contains(key)
    }
}

impl fmt::Debug for DocumentsIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
