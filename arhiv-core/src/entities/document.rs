use std::fmt;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::schema::DataSchema;

use super::{DocumentData, Id, Refs, Revision};

pub const ERASED_DOCUMENT_TYPE: &str = "";

pub type Timestamp = DateTime<Utc>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub id: Id,
    pub rev: Revision,
    pub prev_rev: Revision,
    pub document_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub data: DocumentData,
}

impl Document {
    #[must_use]
    pub fn new(document_type: &str) -> Self {
        Document::new_with_data(document_type, DocumentData::new())
    }

    #[must_use]
    pub fn new_with_data(document_type: &str, data: DocumentData) -> Self {
        let now = Utc::now();

        Document {
            id: Id::new(),
            rev: Revision::STAGING,
            prev_rev: Revision::STAGING,
            document_type: document_type.to_string(),
            created_at: now,
            updated_at: now,
            data,
        }
    }

    #[must_use]
    pub fn is_erased(&self) -> bool {
        self.document_type == ERASED_DOCUMENT_TYPE
    }

    #[must_use]
    pub fn is_staged(&self) -> bool {
        self.rev == Revision::STAGING
    }

    pub fn extract_refs(&self, schema: &DataSchema) -> Result<Refs> {
        schema.extract_refs(&self.document_type, &self.data)
    }

    pub(crate) fn erase(&mut self) {
        self.document_type = ERASED_DOCUMENT_TYPE.to_string();
        self.rev = Revision::STAGING;
        self.prev_rev = Revision::STAGING;
        self.data = DocumentData::new();
        self.updated_at = Utc::now();
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
            "[Document {}/{} {}]",
            self.document_type, self.id, self.rev,
        )
    }
}
