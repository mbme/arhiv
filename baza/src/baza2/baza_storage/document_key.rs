use std::fmt;

use anyhow::{anyhow, Context, Result};

use crate::entities::{Document, Id, Revision};

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct BazaDocumentKey {
    pub id: Id,
    pub rev: Revision,
}

impl BazaDocumentKey {
    pub fn new(id: Id, rev: Revision) -> Self {
        Self { id, rev }
    }

    pub fn for_document(document: &Document) -> Self {
        BazaDocumentKey::new(document.id.clone(), document.rev.clone())
    }

    pub fn parse(value: &str) -> Result<Self> {
        let (id_raw, rev_raw) = value
            .split_once(' ')
            .context(anyhow!("Failed to split value '{value}'"))?;

        let id = Id::from(id_raw);
        let rev = Revision::from_file_name(rev_raw)?;

        Ok(Self { id, rev })
    }

    pub fn serialize(&self) -> String {
        format!("{} {}", self.id, self.rev.to_file_name())
    }
}

impl fmt::Debug for BazaDocumentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("[BazaDocumentKey {}]", &self.serialize()))
    }
}
