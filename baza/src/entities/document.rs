use std::fmt;

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use rs_utils::{now, Timestamp};

use super::{DocumentClass, DocumentData, Id, Revision};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Document<D = DocumentData> {
    pub id: Id,
    pub rev: Option<Revision>,
    #[serde(flatten)]
    pub class: DocumentClass,
    pub updated_at: Timestamp,
    pub data: D,
}

impl<D> Document<D> {
    #[must_use]
    pub fn new_with_data(class: DocumentClass, data: D) -> Self {
        Document {
            id: Id::new(),
            rev: None,
            class,
            updated_at: now(),
            data,
        }
    }
}

impl Document {
    #[must_use]
    pub fn new(class: DocumentClass) -> Self {
        Document::new_with_data(class, DocumentData::new())
    }

    #[must_use]
    pub fn is_erased(&self) -> bool {
        self.class.is_erased()
    }

    #[must_use]
    pub fn is_staged(&self) -> bool {
        self.rev.is_none()
    }

    #[must_use]
    pub fn is_committed(&self) -> bool {
        !self.is_staged()
    }

    pub fn get_rev(&self) -> Result<&Revision> {
        self.rev.as_ref().context("document revision is missing")
    }

    pub(crate) fn erase(&mut self) {
        self.class = DocumentClass::erased();
        self.data = DocumentData::new();
        self.updated_at = now();
    }

    pub(crate) fn stage(&mut self) {
        self.rev = None;
    }

    pub fn convert<D: DeserializeOwned>(self) -> Result<Document<D>> {
        let data: D = serde_json::from_value(self.data.into()).context("failed to convert data")?;

        Ok(Document {
            id: self.id,
            rev: self.rev,
            class: self.class,
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
            class: self.class,
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
            self.class,
            self.id,
            Revision::to_string(&self.rev),
        )
    }
}
