use std::fmt;

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use rs_utils::{now, Timestamp};

use super::{DocumentData, DocumentType, Id, Revision};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Document<D = DocumentData> {
    pub id: Id,
    pub rev: Revision,
    pub prev_rev: Revision,
    #[serde(flatten)]
    pub document_type: DocumentType,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub data: D,
}

impl<D> Document<D> {
    #[must_use]
    pub fn new_with_data(document_type: DocumentType, data: D) -> Self {
        let now = now();

        Document {
            id: Id::new(),
            rev: Revision::STAGING,
            prev_rev: Revision::STAGING,
            document_type,
            created_at: now,
            updated_at: now,
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
    pub fn is_erased(&self) -> bool {
        self.document_type.is_erased()
    }

    #[must_use]
    pub fn is_staged(&self) -> bool {
        self.rev == Revision::STAGING
    }

    pub fn is_initial(&self) -> bool {
        self.prev_rev == Revision::STAGING
    }

    pub fn erase(&mut self) {
        self.document_type = DocumentType::erased();
        self.rev = Revision::STAGING;
        self.prev_rev = Revision::STAGING;
        self.data = DocumentData::new();
        self.updated_at = now();
    }

    pub fn convert<D: DeserializeOwned>(self) -> Result<Document<D>> {
        let data: D = serde_json::from_value(self.data.into()).context("failed to convert data")?;

        Ok(Document {
            id: self.id,
            rev: self.rev,
            prev_rev: self.prev_rev,
            document_type: self.document_type,
            created_at: self.created_at,
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
            prev_rev: self.prev_rev,
            document_type: self.document_type,
            created_at: self.created_at,
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
            self.document_type, self.id, self.rev,
        )
    }
}
