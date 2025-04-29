use std::fmt;

use anyhow::{Context, Result};
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

    pub fn from_string_keys(keys: impl Iterator<Item = String>) -> LinesIndex {
        let mut index = keys.collect::<Vec<_>>();

        index.insert(0, "info".to_string());

        LinesIndex::new(index.into_iter())
    }

    pub fn from_document_keys_refs<'k>(items: impl Iterator<Item = &'k DocumentKey>) -> LinesIndex {
        Self::from_string_keys(items.map(|key| key.serialize()))
    }

    pub fn from_document_keys(items: impl Iterator<Item = DocumentKey>) -> LinesIndex {
        Self::from_string_keys(items.map(|key| key.serialize()))
    }

    pub fn len(&self) -> usize {
        self.0.len()
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
