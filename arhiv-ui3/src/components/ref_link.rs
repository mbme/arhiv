use anyhow::*;
use serde::Serialize;
use serde_json::Value;

use crate::templates::TEMPLATES;
use arhiv_core::{entities::*, Arhiv};
use rs_utils::log;

#[derive(Serialize)]
#[serde(tag = "mode")]
enum RefMode<'a> {
    Ref {
        id: &'a Id,
        document_type: &'a str,
        title: &'a str,
        archived: bool,
    },
    Unknown {
        id: &'a Id,
    },
    Image {
        id: &'a Id,
        title: &'a str,
    },
}

enum RefInfo {
    Id(Id),
    Document(Document),
}

pub struct Ref {
    info: RefInfo,
    preview_attachments: bool,
}

impl Ref {
    pub fn from_id(id: impl Into<Id>) -> Self {
        Ref {
            info: RefInfo::Id(id.into()),
            preview_attachments: false,
        }
    }

    pub fn from_document(document: Document) -> Self {
        Ref {
            info: RefInfo::Document(document),
            preview_attachments: false,
        }
    }

    pub fn preview_attachments(mut self) -> Self {
        self.preview_attachments = true;

        self
    }

    fn get_context(self, arhiv: &Arhiv) -> Result<Value> {
        let document = match self.info {
            RefInfo::Id(ref id) => match arhiv.get_document(id)? {
                Some(document) => document,
                None => {
                    log::warn!("Got broken reference: {}", id);

                    return serde_json::to_value(RefMode::Unknown { id })
                        .context("failed to serialize");
                }
            },
            RefInfo::Document(document) => document,
        };

        if !self.preview_attachments || !Attachment::is_attachment(&document) {
            let title = arhiv.get_schema().get_title(&document)?;

            return serde_json::to_value(RefMode::Ref {
                id: &document.id,
                document_type: &document.document_type,
                title,
                archived: document.archived,
            })
            .context("failed to serialize");
        }

        let attachment = Attachment::from(document)?;
        let title = arhiv.get_schema().get_title(&attachment)?;

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
            archived: attachment.archived,
        })
        .context("failed to serialize")
    }

    pub fn render(self, arhiv: &Arhiv) -> Result<String> {
        let context = self.get_context(&arhiv)?;

        TEMPLATES.render("components/ref_link.html.tera", context)
    }
}
