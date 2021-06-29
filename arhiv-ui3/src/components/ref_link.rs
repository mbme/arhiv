use anyhow::*;
use serde::Serialize;
use serde_json::json;

use crate::templates::TEMPLATES;
use arhiv_core::{entities::Id, schema::SCHEMA, Arhiv};

#[derive(Serialize)]
struct RefOptions {
    render_images: bool,
}

pub struct Ref {
    id: Id,
    options: RefOptions,
}

impl Ref {
    pub fn new(id: impl Into<Id>) -> Self {
        Ref {
            id: id.into(),
            options: RefOptions {
                render_images: false,
            },
        }
    }

    pub fn render_images(mut self) -> Self {
        self.options.render_images = true;

        self
    }

    pub fn render(self, arhiv: &Arhiv) -> Result<String> {
        let document = arhiv.get_document(&self.id)?;

        let context = if let Some(document) = document {
            let title = SCHEMA.get_title(&document)?;

            json!({
                "id": self.id, //
                "unknown": false,
                "document_type": document.document_type,
                "title": title,
            })
        } else {
            json!({
                "id": self.id, //
                "unknown": true,
            })
        };

        TEMPLATES.render("components/ref_link.html.tera", context)
    }
}
