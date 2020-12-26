use std::collections::HashSet;

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
        document_type: row.get("type")?,
        archived: row.get("archived")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        refs: extract_refs(row.get("refs")?)?,
        data: row.get("data")?,
    })
}
