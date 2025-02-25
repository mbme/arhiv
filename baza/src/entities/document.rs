use std::fmt;

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use rs_utils::{now, Timestamp};

use super::{DocumentData, DocumentKey, DocumentType, Id, Revision, ERASED_DOCUMENT_TYPE};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct Document<D = DocumentData> {
    pub id: Id,
    pub rev: Revision,
    pub document_type: DocumentType,
    pub updated_at: Timestamp,
    pub data: D,
}

impl<D> Document<D> {
    #[must_use]
    pub fn new_with_data(document_type: DocumentType, data: D) -> Self {
        Document {
            id: Id::new(),
            rev: Revision::initial(),
            document_type,
            updated_at: now(),
            data,
        }
    }
}

impl Document {
    #[must_use]
    pub fn new(document_type: DocumentType) -> Self {
        Document::new_with_data(document_type, DocumentData::new())
    }

    #[must_use]
    pub fn create_key(&self) -> DocumentKey {
        DocumentKey::for_document(self)
    }

    #[must_use]
    pub fn is_erased(&self) -> bool {
        self.document_type.is(ERASED_DOCUMENT_TYPE)
    }

    #[must_use]
    pub fn is_staged(&self) -> bool {
        self.rev.is_initial()
    }

    #[must_use]
    pub fn is_committed(&self) -> bool {
        !self.is_staged()
    }

    #[cfg(test)]
    pub fn with_id(mut self, id: Id) -> Self {
        self.id = id;

        self
    }

    #[cfg(test)]
    pub fn with_rev(mut self, rev: serde_json::Value) -> Self {
        let revision = Revision::from_value(rev).expect("must be valid revision");
        self.rev = revision;

        self
    }

    #[cfg(test)]
    pub fn with_data(mut self, value: serde_json::Value) -> Self {
        let data = value.try_into().expect("must be valid DocumentData");
        self.data = data;

        self
    }

    pub fn erase(&mut self) {
        self.document_type = DocumentType::erased();
        self.data = DocumentData::new();
        self.updated_at = now();
    }

    pub(crate) fn stage(&mut self) {
        self.rev = Revision::initial();
    }

    pub fn convert<D: DeserializeOwned>(self) -> Result<Document<D>> {
        let data: D = serde_json::from_value(self.data.into()).context("failed to convert data")?;

        Ok(Document {
            id: self.id,
            rev: self.rev,
            document_type: self.document_type,
            updated_at: self.updated_at,
            data,
        })
    }
}

impl<D: Serialize> Document<D> {
    pub fn into_document(self) -> Result<Document> {
        let data: DocumentData = serde_json::to_value(self.data)
            .context("failed to convert to value")?
            .try_into()?;

        Ok(Document {
            id: self.id,
            rev: self.rev,
            document_type: self.document_type,
            updated_at: self.updated_at,
            data,
        })
    }
}

impl std::str::FromStr for Document {
    type Err = anyhow::Error;

    fn from_str(data: &str) -> Result<Document> {
        serde_json::from_str(data).context("Failed to parse document json")
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[Document {} {} {}]",
            self.document_type,
            self.id,
            Revision::to_string(&self.rev),
        )
    }
}
