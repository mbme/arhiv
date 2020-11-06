use super::{Id, Revision};
use anyhow::*;
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Document<T = Value> {
    pub id: Id,
    pub rev: Revision,
    #[serde(rename = "type")]
    pub document_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub refs: HashSet<Id>,
    pub archived: bool,
    pub data: T,
}

impl Document {
    pub fn new<S: Into<String>, T>(document_type: S, data: T) -> Document<T> {
        let now = Utc::now();

        Document {
            id: Id::new(),
            rev: Revision::STAGING,
            document_type: document_type.into(),
            created_at: now,
            updated_at: now,
            refs: HashSet::new(),
            archived: false,
            data,
        }
    }

    pub fn into<K: DeserializeOwned>(self) -> Document<K> {
        Document {
            id: self.id,
            rev: self.rev,
            document_type: self.document_type,
            created_at: self.created_at,
            updated_at: self.updated_at,
            refs: self.refs,
            archived: self.archived,
            data: serde_json::from_value(self.data).expect("must be able to parse data"),
        }
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize document to json")
    }
}

impl<T: Serialize> Document<T> {
    pub fn into_value(self) -> Document {
        Document {
            id: self.id,
            rev: self.rev,
            document_type: self.document_type,
            created_at: self.created_at,
            updated_at: self.updated_at,
            refs: self.refs,
            archived: self.archived,
            data: serde_json::to_value(self.data).expect("must be able to convert to value"),
        }
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
            "[Document {}: {} {}]",
            self.document_type, self.id, self.rev,
        )
    }
}
