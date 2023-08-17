use anyhow::{anyhow, Context, Result};
use rusqlite::Row;
use serde_json::Value;

use crate::entities::{BLOBId, Document, DocumentClass};

pub fn extract_document(row: &Row) -> Result<Document> {
    let document_type: String = row.get("document_type")?;
    let subtype: String = row.get("subtype")?;

    Ok(Document {
        id: row.get("id")?,
        rev: {
            let rev: Value = row.get("rev")?;

            rev.try_into()?
        },
        class: DocumentClass::new(document_type, subtype),
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
