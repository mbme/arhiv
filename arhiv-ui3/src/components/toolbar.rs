use anyhow::*;
use serde::Serialize;
use serde_json::{json, Value};

use crate::templates::TEMPLATES;
use arhiv_core::entities::Document;

#[derive(Serialize)]
pub enum Breadcrumb<'d> {
    Document(&'d Document),
    Collection(String),
    String(String),
}

pub struct Toolbar {
    parent_collection: Option<String>,
    breadcrumbs: Vec<Value>,
    on_close: Option<String>,
    action: Option<(String, String)>,
}

impl Toolbar {
    pub fn new(parent_collection: Option<String>) -> Self {
        Toolbar {
            parent_collection,
            breadcrumbs: vec![],
            on_close: None,
            action: None,
        }
    }

    pub fn with_breadcrubs(mut self, breadcrumbs: Vec<Breadcrumb>) -> Self {
        let breadcrumbs_count = breadcrumbs.len();

        self.breadcrumbs = breadcrumbs
            .into_iter()
            .enumerate()
            .map(|(pos, item)| match item {
                Breadcrumb::Document(document) => {
                    let is_last = pos == breadcrumbs_count - 1;

                    let url = if is_last {
                        "".to_string()
                    } else {
                        format!("/documents/{}", &document.id)
                    };

                    json!({
                        "name": document.document_type.to_uppercase(),
                        "url": url,
                    })
                }
                Breadcrumb::Collection(document_type) => {
                    if let Some(ref collection_id) = self.parent_collection {
                        json!({
                            "name": collection_id,
                            "url": format!("/documents/{}", collection_id),
                        })
                    } else {
                        json!({
                            "name": "CATALOG",
                            "url": format!("/catalogs/{}", document_type),
                        })
                    }
                }
                Breadcrumb::String(name) => {
                    json!({
                        "name": name.to_uppercase(),
                        "url": "",
                    })
                }
            })
            .collect();

        self
    }

    pub fn on_close_document(mut self, document: &Document) -> Self {
        let url = if let Some(ref collection_id) = self.parent_collection {
            format!("/documents/{}", collection_id)
        } else {
            format!("/catalogs/{}", &document.document_type)
        };

        self.on_close = Some(url);

        self
    }

    pub fn on_close(mut self, url: impl Into<String>) -> Self {
        self.on_close = Some(url.into());

        self
    }

    pub fn with_edit(mut self, document: &Document) -> Self {
        let url = if let Some(ref collection_id) = self.parent_collection {
            format!(
                "/documents/{}/edit?parent_collection={}",
                &document.id, collection_id
            )
        } else {
            format!("/documents/{}/edit", &document.id)
        };

        self.action = Some(("Edit".into(), url));

        self
    }

    pub fn render(self) -> Result<String> {
        TEMPLATES.render(
            "components/toolbar.html.tera",
            json!({
                "breadcrumbs": self.breadcrumbs,
                "action": self.action,
                "on_close": self.on_close,
            }),
        )
    }
}
