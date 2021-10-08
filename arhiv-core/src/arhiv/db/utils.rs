use std::convert::TryInto;

use anyhow::*;
use rusqlite::Row;
use serde_json::Value;

use crate::entities::*;

pub fn serialize_refs(refs: &Refs) -> serde_json::Result<String> {
    serde_json::to_string(refs)
}

pub fn extract_document(row: &Row) -> Result<Document> {
    Ok(Document {
        id: row.get("id")?,
        rev: row.get("rev")?,
        prev_rev: row.get("prev_rev")?,
        snapshot_id: row.get("snapshot_id")?,
        document_type: row.get("type")?,
        archived: row.get("archived")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        refs: {
            let value: String = row.get("refs")?;

            serde_json::from_str::<Refs>(&value)?
        },
        data: {
            let data: Value = row.get("data")?;

            data.try_into()?
        },
    })
}

pub fn extract_id(row: &Row) -> Result<Id> {
    row.get("id").context(anyhow!("failed to extract id"))
}
