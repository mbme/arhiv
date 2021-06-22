use anyhow::*;
use serde::Serialize;
use serde_json::json;

use crate::templates::TEMPLATES;
use arhiv_core::{entities::Document, schema::SCHEMA};

#[derive(Serialize)]
struct BreadcrumbItem {
    name: String,
    url: String,
}

#[derive(PartialEq)]
enum Mode {
    Document,
    NewDocument,
    EditDocument,
}

pub struct Breadcrumbs<'doc> {
    document: &'doc Document,
    mode: Mode,
}

impl<'doc> Breadcrumbs<'doc> {
    pub fn new(document: &'doc Document) -> Result<Self> {
        Ok(Breadcrumbs {
            document,
            mode: Mode::Document,
        })
    }

    pub fn for_document_editor(mut self) -> Self {
        self.mode = Mode::EditDocument;

        self
    }

    pub fn for_new_document(mut self) -> Self {
        self.mode = Mode::NewDocument;

        self
    }

    pub fn render(self) -> Result<String> {
        let mut items = vec![];

        if let Some(collection_type) = SCHEMA.get_collection_type(&self.document.document_type) {
            let collection_id = self
                .document
                .get_field_str(collection_type)
                .ok_or(anyhow!("collection field '{}' missing", collection_type))?;

            items.push(BreadcrumbItem {
                name: format!("{} {}", collection_type.to_uppercase(), collection_id),
                url: format!("/documents/{}", collection_id),
            });

            items.push(BreadcrumbItem {
                name: format!(
                    "{} {}",
                    self.document.document_type.to_uppercase(),
                    &self.document.id
                ),
                url: format!("/documents/{}", &self.document.id),
            });
        } else {
            items.push(BreadcrumbItem {
                name: format!("{}S", self.document.document_type.to_uppercase()),
                url: format!("/catalogs/{}", &self.document.document_type),
            });

            items.push(BreadcrumbItem {
                name: self.document.id.to_string(),
                url: format!("/documents/{}", &self.document.id),
            });
        }

        if self.mode == Mode::EditDocument {
            items.push(BreadcrumbItem {
                name: "editor".to_uppercase(),
                url: "".to_string(),
            });
        }

        if self.mode == Mode::NewDocument {
            items.pop();

            items.push(BreadcrumbItem {
                name: format!("NEW {}", self.document.document_type.to_uppercase()),
                url: "".to_string(),
            });
        }

        TEMPLATES.render(
            "components/breadcrumbs.html.tera",
            json!({
                "items": items,
            }),
        )
    }
}
