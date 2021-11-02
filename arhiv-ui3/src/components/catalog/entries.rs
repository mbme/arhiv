use anyhow::*;
use serde_json::{json, Value};

use arhiv_core::{entities::*, markup::MarkupStr, Arhiv};

use crate::{
    markup::MarkupStringExt, template_fn, ui_config::get_catalog_config, urls::document_url,
};

pub struct CatalogConfig {
    pub preview: Option<&'static str>,
    pub fields: Vec<&'static str>,
}

impl Default for CatalogConfig {
    fn default() -> Self {
        CatalogConfig {
            preview: None,
            fields: vec![],
        }
    }
}

template_fn!(render_template, "./entries.html.tera");

pub struct CatalogEntries {
    pub parent_collection: Option<Id>,
    pub show_type: bool,
    pub show_id: bool,
}

impl CatalogEntries {
    pub fn new() -> Self {
        CatalogEntries {
            parent_collection: None,
            show_type: false,
            show_id: false,
        }
    }

    fn prepare_entry_data(&self, document: &Document, arhiv: &Arhiv) -> Result<Value> {
        let document_type = &document.document_type;
        let config = get_catalog_config(document_type);

        let title = arhiv.get_schema().get_title(document)?;

        let mut preview = None;

        if let Some(preview_field) = config.preview {
            let markup: MarkupStr = document
                .data
                .get_str(preview_field)
                .ok_or_else(|| anyhow!("preview field missing"))?
                .into();

            preview = Some(markup.preview(4).to_html(arhiv));
        }

        let fields: Vec<_> = config
            .fields
            .iter()
            .map(|field| {
                (
                    *field,
                    document.data.get_str(field).unwrap_or_default().to_string(),
                )
            })
            .collect();

        let url = document_url(&document.id, &self.parent_collection);

        Ok(json!({
            "id": document.id,
            "url": url,
            "title": title,
            "document_type": document_type,
            "preview": preview,
            "fields": fields,
        }))
    }

    pub fn render(&self, documents: &[&Document], arhiv: &Arhiv) -> Result<String> {
        let entries = documents
            .iter()
            .map(|document| self.prepare_entry_data(document, arhiv))
            .collect::<Result<Vec<_>>>()?;

        render_template(json!({
            "entries": entries,
            "show_type": self.show_type,
            "show_id": self.show_id,
        }))
    }
}
