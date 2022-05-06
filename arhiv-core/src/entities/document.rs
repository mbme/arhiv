use std::fmt;

use anyhow::{ensure, Context, Result};
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::schema::DataSchema;

use super::{DocumentData, Id, Refs, Revision};

pub const ERASED_DOCUMENT_TYPE: &str = "";

pub type Timestamp = DateTime<Utc>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Document<D = DocumentData> {
    pub id: Id,
    pub rev: Revision,
    pub prev_rev: Revision,
    pub document_type: String,
    pub subtype: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub data: D,
}

impl<D> Document<D> {
    #[must_use]
    pub fn new_with_data(document_type: &str, subtype: &str, data: D) -> Self {
        let now = Utc::now();

        Document {
            id: Id::new(),
            rev: Revision::STAGING,
            prev_rev: Revision::STAGING,
            document_type: document_type.to_string(),
            subtype: subtype.to_string(),
            created_at: now,
            updated_at: now,
            data,
        }
    }
}

impl Document {
    #[must_use]
    pub fn new(document_type: &str, subtype: &str) -> Self {
        Document::new_with_data(document_type, subtype, DocumentData::new())
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
        schema.extract_refs(&self.document_type, &self.subtype, &self.data)
    }

    pub fn erase(&mut self) {
        self.document_type = ERASED_DOCUMENT_TYPE.to_string();
        self.subtype = "".to_string();
        self.rev = Revision::STAGING;
        self.prev_rev = Revision::STAGING;
        self.data = DocumentData::new();
        self.updated_at = Utc::now();
    }

    pub fn ensure_document_type(&self, document_type: &str) -> Result<()> {
        ensure!(
            self.document_type == document_type,
            "expected document_type to be '{}', got '{}' instead",
            self.document_type,
            document_type
        );

        Ok(())
    }

    pub fn convert<D: DeserializeOwned>(self) -> Result<Document<D>> {
        let data: D = serde_json::from_value(self.data.into()).context("failed to convert data")?;

        Ok(Document {
            id: self.id,
            rev: self.rev,
            prev_rev: self.prev_rev,
            document_type: self.document_type,
            subtype: self.subtype,
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
            subtype: self.subtype,
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
            "[Document {}/{} {} {}]",
            self.document_type, self.subtype, self.id, self.rev,
        )
    }
}
