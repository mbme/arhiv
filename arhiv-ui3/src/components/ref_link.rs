use anyhow::*;
use serde::Serialize;
use serde_json::Value;

use crate::{template_fn, urls::document_url};
use arhiv_core::{entities::*, Arhiv};
use rs_utils::log;

template_fn!(render_template, "./ref_link.html.tera");

#[derive(Serialize)]
#[serde(tag = "mode")]
enum RefMode<'a> {
    Ref {
        document_type: &'a str,
        title: String,
        url: String,
    },
    Unknown {
        id: &'a Id,
        url: String,
    },
    Image {
        id: &'a Id,
        title: String,
        url: String,
    },
}

enum RefInfo {
    Id(Id),
    Document(Box<Document>),
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
            info: RefInfo::Document(Box::new(document)),
            preview_attachments: false,
        }
    }

    pub fn preview_attachments(mut self) -> Self {
        self.preview_attachments = true;

        self
    }

    fn get_context(self, arhiv: &Arhiv) -> Result<Value> {
        let document = match self.info {
            RefInfo::Id(ref id) => {
                if let Some(document) = arhiv.get_document(id)? {
                    document
                } else {
                    log::warn!("Got broken reference: {}", id);

                    return serde_json::to_value(RefMode::Unknown {
                        id,
                        url: document_url(id, &None),
                    })
                    .context("failed to serialize");
                }
            }
            RefInfo::Document(document) => *document,
        };

        if !self.preview_attachments || !Attachment::is_attachment(&document) {
            let title = arhiv.get_schema().get_title(&document)?;

            return serde_json::to_value(RefMode::Ref {
                document_type: &document.document_type,
                title,
                url: document_url(&document.id, &None),
            })
            .context("failed to serialize");
        }

        let attachment = Attachment::from(document)?;
        let title = arhiv.get_schema().get_title(&attachment)?;

        if attachment.is_image() {
            return serde_json::to_value(RefMode::Image {
                id: &attachment.id,
                title,
                url: document_url(&attachment.id, &None),
            })
            .context("failed to serialize");
        }

        serde_json::to_value(RefMode::Ref {
            document_type: &attachment.document_type,
            title,
            url: document_url(&attachment.id, &None),
        })
        .context("failed to serialize")
    }

    pub fn render(self, arhiv: &Arhiv) -> Result<String> {
        let context = self.get_context(arhiv)?;

        render_template(context)
    }
}
