use std::fmt;

use anyhow::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::schema::DataSchema;

use super::{DocumentData, Id, Refs, Revision, SnapshotId};

pub const ERASED_DOCUMENT_TYPE: &str = "";

pub type Timestamp = DateTime<Utc>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Document {
    pub id: Id,
    pub rev: Revision,
    pub prev_rev: Revision,
    pub document_type: String,
    pub snapshot_id: SnapshotId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub data: DocumentData,
}

impl Document {
    pub fn new(document_type: impl Into<String>) -> Self {
        Document::new_with_data(document_type, DocumentData::new())
    }

    pub fn new_with_data(document_type: impl Into<String>, data: DocumentData) -> Self {
        let now = Utc::now();

        Document {
            id: Id::new(),
            rev: Revision::STAGING,
            prev_rev: Revision::STAGING,
            snapshot_id: SnapshotId::new(),
            document_type: document_type.into(),
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
