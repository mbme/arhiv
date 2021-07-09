use anyhow::*;
use serde::Serialize;
use serde_json::json;

use crate::templates::TEMPLATES;
use arhiv_core::entities::Document;

#[derive(Serialize)]
pub struct Breadcrumb {
    name: String,
    url: String,
}

impl Breadcrumb {
    pub fn for_document(document: &Document, link: bool) -> Self {
        Breadcrumb {
            name: document.document_type.to_uppercase(),
            url: if link {
                format!("/documents/{}", &document.id)
            } else {
                "".to_string()
            },
        }
    }

    pub fn for_document_collection(
        document: &Document,
        collection_type: Option<&'static str>,
    ) -> Result<Self> {
        if let Some(collection_type) = collection_type {
            let collection_id = document
                .data
                .get_str(collection_type)
                .ok_or(anyhow!("collection field '{}' missing", collection_type))?;

            Ok(Breadcrumb {
                name: collection_type.to_uppercase(),
                url: format!("/documents/{}", collection_id),
            })
        } else {
            Ok(Breadcrumb {
                name: "CATALOG".to_string(),
                url: format!("/catalogs/{}", &document.document_type),
            })
        }
    }

    pub fn for_string(s: impl Into<String>) -> Self {
        Breadcrumb {
            name: s.into().to_uppercase(),
            url: "".to_string(),
        }
    }
}

pub struct Toolbar {
    breadcrumbs: Vec<Breadcrumb>,
    on_close: Option<String>,
    action: Option<(String, String)>,
}

impl Toolbar {
    pub fn new() -> Self {
        Toolbar {
            breadcrumbs: vec![],
            on_close: None,
            action: None,
        }
    }

    pub fn with_breadcrubs(mut self, breadcrumbs: Vec<Breadcrumb>) -> Self {
        self.breadcrumbs = breadcrumbs;

        self
    }

    pub fn on_close_document(
        mut self,
        document: &Document,
        collection_type: Option<&'static str>,
    ) -> Self {
        let url = if let Some(collection_type) = collection_type {
            let collection_id = document.data.get_mandatory_str(collection_type);

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

    pub fn with_action(mut self, name: impl Into<String>, href: impl Into<String>) -> Self {
        self.action = Some((name.into(), href.into()));

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
