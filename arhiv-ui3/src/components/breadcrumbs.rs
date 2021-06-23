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

impl BreadcrumbItem {
    pub fn for_document(document: &Document, link: bool) -> Self {
        BreadcrumbItem {
            name: document.document_type.to_uppercase(),
            url: if link {
                format!("/documents/{}", &document.id)
            } else {
                "".to_string()
            },
        }
    }

    pub fn for_document_collection(document: &Document) -> Result<Self> {
        if let Some(collection_type) = SCHEMA.get_collection_type(&document.document_type) {
            let collection_id = document
                .get_field_str(collection_type)
                .ok_or(anyhow!("collection field '{}' missing", collection_type))?;

            Ok(BreadcrumbItem {
                name: collection_type.to_uppercase(),
                url: format!("/documents/{}", collection_id),
            })
        } else {
            Ok(BreadcrumbItem {
                name: format!("{}S", document.document_type.to_uppercase()),
                url: format!("/catalogs/{}", &document.document_type),
            })
        }
    }

    pub fn for_string(s: impl Into<String>) -> Self {
        BreadcrumbItem {
            name: s.into().to_uppercase(),
            url: "".to_string(),
        }
    }
}

pub enum Breadcrumbs<'doc> {
    Index,
    Catalog(String),
    Document(&'doc Document),
    NewDocumentVariants,
    NewDocument(&'doc Document),
    DocumentEditor(&'doc Document),
}

impl<'doc> Breadcrumbs<'doc> {
    pub fn render(self) -> Result<String> {
        let items = match self {
            Breadcrumbs::Index => {
                vec![BreadcrumbItem::for_string("index")]
            }
            Breadcrumbs::Document(document) => {
                vec![
                    BreadcrumbItem::for_document_collection(document)?,
                    BreadcrumbItem::for_document(document, false),
                ]
            }
            Breadcrumbs::DocumentEditor(document) => {
                vec![
                    BreadcrumbItem::for_document_collection(document)?,
                    BreadcrumbItem::for_document(document, true),
                    BreadcrumbItem::for_string("editor"),
                ]
            }
            Breadcrumbs::NewDocumentVariants => {
                vec![BreadcrumbItem::for_string("new document")]
            }
            Breadcrumbs::NewDocument(document) => {
                vec![
                    BreadcrumbItem::for_document_collection(document)?,
                    BreadcrumbItem::for_string(format!("new {}", document.document_type)),
                ]
            }
            Breadcrumbs::Catalog(document_type) => {
                vec![BreadcrumbItem::for_string(format!("{}s", document_type))]
            }
        };

        TEMPLATES.render(
            "components/breadcrumbs.html.tera",
            json!({
                "items": items,
            }),
        )
    }
}
