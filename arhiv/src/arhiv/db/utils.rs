use rusqlite::types::ToSql;
use std::collections::HashSet;
use std::{collections::HashMap, rc::Rc};

use crate::entities::*;
use anyhow::*;
use rusqlite::Row;

fn extract_refs(value: String) -> serde_json::Result<HashSet<Id>> {
    serde_json::from_str::<HashSet<Id>>(&value)
}

pub fn serialize_refs(refs: &HashSet<Id>) -> serde_json::Result<String> {
    serde_json::to_string(&refs)
}

pub fn extract_document(row: &Row) -> Result<Document> {
    Ok(Document {
        id: row.get("id")?,
        rev: row.get("rev")?,
        snapshot_id: row.get("snapshot_id")?,
        document_type: row.get("type")?,
        archived: row.get("archived")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        refs: extract_refs(row.get("refs")?)?,
        data: row.get("data")?,
    })
}

pub fn extract_document_history(row: &Row) -> Result<DocumentHistory> {
    Ok(DocumentHistory::new(
        extract_document(row)?,
        row.get("base_rev")?,
    ))
}

pub struct Params {
    params: HashMap<String, Rc<dyn ToSql>>,
}

impl Params {
    pub fn new() -> Self {
        Params {
            params: HashMap::new(),
        }
    }

    pub fn insert<S: Into<String>>(&mut self, key: S, value: Rc<dyn ToSql>) {
        self.params.insert(key.into(), value);
    }

    pub fn get(&self) -> Vec<(&str, &dyn ToSql)> {
        let mut params: Vec<(&str, &dyn ToSql)> = vec![];
        for (key, value) in self.params.iter() {
            params.push((key, value.as_ref()));
        }

        params
    }
}
