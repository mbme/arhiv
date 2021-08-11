use anyhow::*;
use serde::Serialize;
use serde_json::json;

use crate::templates::TEMPLATES;
use arhiv_core::{entities::Document, Arhiv};

#[derive(Serialize)]
pub enum Breadcrumb<'d> {
    Document(&'d Document),
    Collection(String),
    String(String),
}

pub struct Toolbar<'d> {
    parent_collection: Option<String>,
    breadcrumbs: Vec<Breadcrumb<'d>>,
    on_close: Option<String>,
    action: Option<(String, String)>,
}

impl<'d> Toolbar<'d> {
    pub fn new(parent_collection: Option<String>) -> Self {
        Toolbar {
            parent_collection,
            breadcrumbs: vec![],
            on_close: None,
            action: None,
        }
    }

    pub fn with_breadcrumb(mut self, breadcrumb: Breadcrumb<'d>) -> Self {
        self.breadcrumbs.push(breadcrumb);

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

    pub fn render(self, arhiv: &Arhiv) -> Result<String> {
        let breadcrumbs_count = self.breadcrumbs.len();
        let parent_collection = self.parent_collection;

        let breadcrumbs = self
            .breadcrumbs
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

                    Ok(json!({
                        "name": document.document_type.to_uppercase(),
                        "url": url,
                    }))
                }
                Breadcrumb::Collection(document_type) => {
                    if let Some(ref collection_id) = parent_collection {
                        let document = arhiv
                            .get_document(collection_id)?
                            .ok_or(anyhow!("can't find parent collection"))?;

                        let name = arhiv.schema.get_title(&document)?;

                        Ok(json!({
                            "name": format!("{} {}", document.document_type.to_uppercase(), name),
                            "url": format!("/documents/{}", collection_id),
                        }))
                    } else {
                        Ok(json!({
                            "name": "CATALOG",
                            "url": format!("/catalogs/{}", document_type),
                        }))
                    }
                }
                Breadcrumb::String(name) => Ok(json!({
                    "name": name.to_uppercase(),
                    "url": "",
                })),
            })
            .collect::<Result<Vec<_>>>()?;

        TEMPLATES.render(
            "components/toolbar.html.tera",
            json!({
                "breadcrumbs": breadcrumbs,
                "action": self.action,
                "on_close": self.on_close,
            }),
        )
    }
}
