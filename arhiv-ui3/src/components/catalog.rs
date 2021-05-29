use anyhow::*;
use chrono::{DateTime, Local};
use serde::Serialize;
use serde_json::{json, Value};

use arhiv::{entities::*, markup::MarkupRenderer};

#[derive(Serialize)]
struct CatalogEntry {
    id: Id,
    document_type: String,
    preview: String,
    updated_at: DateTime<Local>,
}

pub fn prepare_catalog_values(
    renderer: &MarkupRenderer,
    documents: Vec<Document>,
) -> Result<Value> {
    let items: Vec<_> = documents
        .into_iter()
        .map(|document| CatalogEntry {
            preview: renderer
                .get_preview(&document)
                .unwrap_or("No preview".to_string()),
            id: document.id,
            document_type: document.document_type,
            updated_at: document.updated_at.into(),
        })
        .collect();

    Ok(json!({
        "items": items,
    }))
}
