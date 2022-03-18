use anyhow::{anyhow, Context, Result};
use rusqlite::Row;
use serde_json::Value;

use crate::entities::{BLOBId, Document};

pub fn extract_document(row: &Row) -> Result<Document> {
    Ok(Document {
        id: row.get("id")?,
        rev: row.get("rev")?,
        prev_rev: row.get("prev_rev")?,
        document_type: row.get("type")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        data: {
            let data: Value = row.get("data")?;

            data.try_into()?
        },
    })
}

pub fn extract_blob_id(row: &Row) -> Result<BLOBId> {
    row.get("blob_id")
        .context(anyhow!("failed to extract blob_id"))
}
