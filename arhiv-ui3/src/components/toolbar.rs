use anyhow::*;
use serde::Serialize;
use serde_json::json;

use crate::{
    template_fn,
    urls::{catalog_url, document_editor_url, document_url, parent_collection_url, NewDocumentUrl},
};
use arhiv_core::{
    entities::{Document, Id},
    Arhiv,
};

template_fn!(render_template, "./toolbar.html.tera");

#[derive(Serialize)]
pub enum Breadcrumb<'d> {
    Document(&'d Document),
    Collection(String),
    String(String),
}

#[derive(Serialize)]
struct Action {
    url: String,
    name: String,
    icon_id: Option<&'static str>,
}

pub struct Toolbar<'d> {
    parent_collection: Option<Id>,
    breadcrumbs: Vec<Breadcrumb<'d>>,
    on_close: Option<String>,
    actions: Vec<Action>,
}

impl<'d> Toolbar<'d> {
    pub fn new(parent_collection: Option<Id>) -> Self {
        Toolbar {
            parent_collection,
            breadcrumbs: vec![],
            on_close: None,
            actions: vec![],
        }
    }

    pub fn with_breadcrumb(mut self, breadcrumb: Breadcrumb<'d>) -> Self {
        self.breadcrumbs.push(breadcrumb);

        self
    }

    pub fn on_close_document(mut self, document: &Document) -> Self {
        let url = parent_collection_url(&document.document_type, &self.parent_collection);

        self.on_close = Some(url);

        self
    }

    pub fn on_close(mut self, url: impl Into<String>) -> Self {
        self.on_close = Some(url.into());

        self
    }

    pub fn with_edit(mut self, document: &Document) -> Self {
        let url = document_editor_url(&document.id, &self.parent_collection);

        self.actions.push(Action {
            name: "Edit".to_string(),
            url,
            icon_id: Some("icon-document-edit"),
        });

        self
    }

    pub fn with_new_collection_item(mut self, document_type: &str, field: &str, id: &Id) -> Self {
        let url = NewDocumentUrl::CollectionItem(document_type, field, id).build();

        self.actions.push(Action {
            name: document_type.to_string(),
            url,
            icon_id: Some("icon-document-add"),
        });

        self
    }

    pub fn with_new_document(mut self, document_type: &str) -> Self {
        let url = NewDocumentUrl::Document(document_type).build();

        self.actions.push(Action {
            name: document_type.to_string(),
            url,
            icon_id: Some("icon-document-add"),
        });

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
                        document_url(&document.id, &None)
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
                            .ok_or_else(|| anyhow!("can't find parent collection"))?;

                        let name = arhiv.get_schema().get_title(&document)?;

                        Ok(json!({
                            "name": format!("{} {}", document.document_type.to_uppercase(), name),
                            "url": document_url(collection_id, &None),
                        }))
                    } else {
                        Ok(json!({
                            "name": "CATALOG",
                            "url": catalog_url(&document_type),
                        }))
                    }
                }
                Breadcrumb::String(name) => Ok(json!({
                    "name": name.to_uppercase(),
                    "url": "",
                })),
            })
            .collect::<Result<Vec<_>>>()?;

        render_template(json!({
            "breadcrumbs": breadcrumbs,
            "actions": self.actions,
            "on_close": self.on_close,
        }))
    }
}
