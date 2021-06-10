use anyhow::*;
use chrono::{DateTime, Local};
use serde::Serialize;
use serde_json::json;

use crate::app_context::AppContext;
use arhiv::entities::*;

#[derive(Serialize)]
struct CatalogEntry {
    id: Id,
    document_type: String,
    preview: String,
    updated_at: DateTime<Local>,
}

pub struct Catalog {
    documents: Vec<Document>,
    pattern: String,
}

impl Catalog {
    pub fn new(documents: Vec<Document>, pattern: String) -> Self {
        Catalog { documents, pattern }
    }

    pub fn render(self, context: &AppContext) -> Result<String> {
        let items: Vec<_> = self
            .documents
            .into_iter()
            .map(|document| CatalogEntry {
                preview: context.render_preview(&document),
                id: document.id,
                document_type: document.document_type,
                updated_at: document.updated_at.into(),
            })
            .collect();

        context.render_template(
            "components/catalog.html.tera",
            json!({
                "items": items,
                "pattern": self.pattern,
            }),
        )
    }
}
