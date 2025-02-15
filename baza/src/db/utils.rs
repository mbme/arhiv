use std::{collections::HashSet, fs};

use anyhow::{anyhow, ensure, Context, Result};
use rusqlite::Row;
use serde_json::Value;

use crate::entities::{BLOBId, Document, DocumentType, Revision};

pub fn extract_document(row: &Row) -> Result<Document> {
    let document_type: String = row.get("document_type")?;

    Ok(Document {
        id: row.get("id")?,
        rev: {
            let rev: Value = row.get("rev")?;

            Revision::from_value(rev).context("failed to parse document rev")?
        },
        document_type: DocumentType::new(document_type),
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

pub fn get_local_blob_ids(dir: &str, trim_ext: &str) -> Result<HashSet<BLOBId>> {
    let items = fs::read_dir(dir)?
        .map(|item| {
            let entry = item.context("Failed to read data entry")?;

            let entry_path = entry.path();

            ensure!(
                entry_path.is_file(),
                "{} isn't a file",
                entry_path.to_string_lossy()
            );

            entry_path
                .file_name()
                .ok_or_else(|| anyhow!("Failed to read file name"))
                .map(|value| {
                    value
                        .to_string_lossy()
                        .trim_end_matches(trim_ext)
                        .to_string()
                })
                .and_then(BLOBId::from_string)
        })
        .collect::<Result<HashSet<_>>>()?;

    Ok(items)
}
