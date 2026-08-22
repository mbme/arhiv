use std::fmt;

use anyhow::{Context, Result, bail, ensure};
use ordermap::OrderSet;

use baza_storage::LinesIndex;

use super::DocumentKey;

/// Document keys in their current storage order.
///
/// Readers preserve non-canonical v1 order for index-to-line association;
/// writers sort the index before emitting a new container.
pub struct DocumentsIndex(OrderSet<DocumentKey>);

impl DocumentsIndex {
    /// Parses an index that starts with `info`, preserving its document-key order.
    pub fn parse(index: &LinesIndex) -> Result<Self> {
        ensure!(
            index.iter().next() == Some("info"),
            "storage index must start with info"
        );

        let keys = index
            .iter()
            .skip(1)
            .map(DocumentKey::parse)
            .collect::<Result<Vec<_>>>()
            .context("Failed to parse DocumentKey")?;

        Self::from_document_keys(keys)
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

    pub fn sort_by_document_key(&mut self) {
        self.0.sort_by(DocumentKey::canonical_cmp);
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

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use baza_storage::LinesIndex;
    use serde_json::json;

    use crate::entities::{DocumentKey, Id, Revision};

    use super::DocumentsIndex;

    #[test]
    fn rejects_index_without_info_as_first_entry() -> Result<()> {
        let index = LinesIndex::from(&["not-info"][..]);

        let error = DocumentsIndex::parse(&index).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("storage index must start with info"),
            "unexpected error: {error:#}"
        );

        Ok(())
    }

    #[test]
    fn rejects_keys_that_collide_after_revision_normalization() -> Result<()> {
        let index = LinesIndex::from(&["info", "aaaaaaaaaaaaaa a:1", "aaaaaaaaaaaaaa a:1-b:0"][..]);

        let error = DocumentsIndex::parse(&index).unwrap_err();
        assert!(
            error.to_string().contains("duplicate document key"),
            "unexpected error: {error:#}"
        );

        Ok(())
    }

    #[test]
    fn sorts_revision_cycle_by_canonical_order() -> Result<()> {
        let id = Id::from("aaaaaaaaaaaaaa");
        let key_b = DocumentKey::new(id.clone(), Revision::from_value(json!({ "b": 1 }))?);
        let key_c = DocumentKey::new(id.clone(), Revision::from_value(json!({ "c": 1 }))?);
        let key_ac = DocumentKey::new(id, Revision::from_value(json!({ "a": 1, "c": 2 }))?);
        let mut index =
            DocumentsIndex::from_document_keys(vec![key_b.clone(), key_c.clone(), key_ac.clone()])?;

        index.sort_by_document_key();

        assert_eq!(
            index.iter().collect::<Vec<_>>(),
            vec![&key_ac, &key_b, &key_c]
        );

        Ok(())
    }
}
