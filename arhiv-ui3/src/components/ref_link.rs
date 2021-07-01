use anyhow::*;
use serde::Serialize;
use serde_json::Value;

use crate::templates::TEMPLATES;
use arhiv_core::{
    entities::{Attachment, Id},
    schema::SCHEMA,
    Arhiv,
};
use rs_utils::log;

#[derive(Serialize)]
#[serde(tag = "mode")]
enum RefMode<'a> {
    Ref {
        id: &'a Id,
        document_type: &'a str,
        title: &'a str,
    },
    Unknown {
        id: &'a Id,
    },
    Image {
        id: &'a Id,
        title: &'a str,
    },
}

pub struct Ref {
    id: Id,
    preview_attachments: bool,
}

impl Ref {
    pub fn new(id: impl Into<Id>) -> Self {
        Ref {
            id: id.into(),
            preview_attachments: false,
        }
    }

    pub fn preview_attachments(&mut self) -> &mut Self {
        self.preview_attachments = true;

        self
    }

    fn get_context(&self, arhiv: &Arhiv) -> Result<Value> {
        let document = {
            match arhiv.get_document(&self.id)? {
                Some(document) => document,
                None => {
                    log::warn!("Got broken reference: {}", &self.id);

                    return serde_json::to_value(RefMode::Unknown { id: &self.id })
                        .context("failed to serialize");
                }
            }
        };

        if !self.preview_attachments || !Attachment::is_attachment(&document) {
            let title = SCHEMA.get_title(&document)?;

            return serde_json::to_value(RefMode::Ref {
                id: &document.id,
                document_type: &document.document_type,
                title,
            })
            .context("failed to serialize");
        }

        let attachment = Attachment::from(document)?;
        let title = SCHEMA.get_title(&attachment)?;

        if attachment.is_image() {
            return serde_json::to_value(RefMode::Image {
                id: &attachment.id,
                title,
            })
            .context("failed to serialize");
        }

        serde_json::to_value(RefMode::Ref {
            id: &attachment.id,
            document_type: &attachment.document_type,
            title,
        })
        .context("failed to serialize")
    }

    pub fn render(&self, arhiv: &Arhiv) -> Result<String> {
        let context = self.get_context(&arhiv)?;

        TEMPLATES.render("components/ref_link.html.tera", context)
    }
}
