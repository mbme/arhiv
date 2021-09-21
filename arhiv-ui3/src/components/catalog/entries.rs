use anyhow::*;
use serde::Serialize;

use arhiv_core::{entities::*, markup::MarkupStr, Arhiv};

use crate::{markup::MarkupStringExt, ui_config::get_catalog_config, urls::document_url};

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

#[derive(Serialize)]
pub struct CatalogEntry {
    url: String,
    document_type: String,
    title: String,
    preview: Option<String>,
    fields: Vec<(&'static str, String)>,
}

impl CatalogEntry {
    pub fn new(document: Document, arhiv: &Arhiv, parent_collection: &Option<Id>) -> Result<Self> {
        let config = get_catalog_config(&document.document_type);

        let data_description = arhiv
            .get_schema()
            .get_data_description(&document.document_type)?;

        let title_field = data_description.pick_title_field()?;

        let title = document
            .data
            .get_str(title_field.name)
            .ok_or_else(|| anyhow!("title field missing"))?;

        let mut preview = None;

        if let Some(preview_field) = config.preview {
            let markup: MarkupStr = document
                .data
                .get_str(preview_field)
                .ok_or_else(|| anyhow!("preview field missing"))?
                .into();

            preview = Some(markup.preview(4).to_html(arhiv));
        }

        let fields = config
            .fields
            .iter()
            .map(|field| {
                (
                    *field,
                    document.data.get_str(field).unwrap_or_default().to_string(),
                )
            })
            .collect();

        Ok(CatalogEntry {
            url: document_url(&document.id, parent_collection),
            title: title.to_string(),
            document_type: document.document_type,
            preview,
            fields,
        })
    }
}
