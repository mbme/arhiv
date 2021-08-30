use anyhow::*;
use serde::Serialize;

use super::config::CatalogConfig;
use crate::markup::MarkupStringExt;
use arhiv_core::{entities::*, markup::MarkupStr, Arhiv};

#[derive(Serialize)]
pub struct CatalogEntry {
    id: Id,
    document_type: String,
    title: String,
    preview: Option<String>,
    fields: Vec<(&'static str, String)>,
}

impl CatalogEntry {
    pub fn new(document: Document, arhiv: &Arhiv, config: &CatalogConfig) -> Result<Self> {
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
                    document.data.get_str(field).unwrap_or("").to_string(),
                )
            })
            .collect();

        Ok(CatalogEntry {
            title: title.to_string(),
            id: document.id,
            document_type: document.document_type,
            preview,
            fields,
        })
    }
}
