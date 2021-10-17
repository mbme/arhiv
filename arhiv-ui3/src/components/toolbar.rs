use anyhow::*;
use serde::Serialize;
use serde_json::json;

use arhiv_core::{
    entities::{Document, Id},
    Arhiv,
};

use crate::{
    template_fn,
    urls::{catalog_url, document_editor_url, document_url, NewDocumentUrl},
};

template_fn!(render_template, "./toolbar.html.tera");

#[derive(Serialize)]
pub struct Breadcrumb {
    url: String,
    name: String,
}

impl Breadcrumb {
    pub fn for_document(document: &Document) -> Self {
        Breadcrumb {
            url: document_url(&document.id, &None),
            name: document.document_type.to_uppercase(),
        }
    }

    pub fn for_collection(
        document: &Document,
        collection_id: &Option<Id>,
        arhiv: &Arhiv,
    ) -> Result<Self> {
        if let Some(ref collection_id) = collection_id {
            let document = arhiv
                .get_document(collection_id)?
                .ok_or_else(|| anyhow!("can't find parent collection"))?;

            let name = arhiv.get_schema().get_title(&document)?;

            Ok(Breadcrumb {
                name: format!("{} {}", document.document_type.to_uppercase(), name),
                url: document_url(collection_id, &None),
            })
        } else {
            Ok(Breadcrumb {
                name: "CATALOG".to_string(),
                url: catalog_url(&document.document_type),
            })
        }
    }

    pub fn string(name: impl Into<String>) -> Self {
        Breadcrumb {
            name: name.into(),
            url: "".to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct Action {
    url: String,
    name: String,
    icon_id: Option<&'static str>,
}

impl Action {
    pub fn edit(document: &Document, parent_id: &Option<Id>) -> Self {
        let url = document_editor_url(&document.id, parent_id);

        Action {
            name: "Edit".to_string(),
            url,
            icon_id: Some("icon-document-edit"),
        }
    }

    pub fn new_document(document_type: &str) -> Self {
        let url = NewDocumentUrl::Document(document_type).build();

        Action {
            name: document_type.to_string(),
            url,
            icon_id: Some("icon-document-add"),
        }
    }

    pub fn new_collection_item(item_type: &str, field: &str, id: &Id) -> Self {
        let url = NewDocumentUrl::CollectionItem(item_type, field, id).build();

        Action {
            name: item_type.to_string(),
            url,
            icon_id: Some("icon-document-add"),
        }
    }
}

pub struct Toolbar {
    breadcrumbs: Vec<Breadcrumb>,
    on_close: Option<String>,
    actions: Vec<Action>,
}

impl Toolbar {
    pub fn new() -> Self {
        Toolbar {
            breadcrumbs: vec![],
            on_close: None,
            actions: vec![],
        }
    }

    pub fn with_breadcrumb(mut self, breadcrumb: Breadcrumb) -> Self {
        self.breadcrumbs.push(breadcrumb);

        self
    }

    pub fn on_close(mut self, url: impl Into<String>) -> Self {
        self.on_close = Some(url.into());

        self
    }

    pub fn with_action(mut self, action: Action) -> Self {
        self.actions.push(action);

        self
    }

    pub fn render(self) -> Result<String> {
        render_template(json!({
            "breadcrumbs": self.breadcrumbs,
            "actions": self.actions,
            "on_close": self.on_close,
        }))
    }
}
