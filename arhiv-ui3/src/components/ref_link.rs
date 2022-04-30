use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;

use arhiv_core::{definitions::Attachment, entities::*, Arhiv};
use rs_utils::log;

use crate::{
    template_fn,
    urls::{blob_url, document_url},
};

template_fn!(render_template, "./ref_link.html.tera");

#[derive(Serialize)]
#[serde(tag = "mode")]
enum RefMode<'a> {
    Ref {
        document_type: &'a str,
        subtype: &'a str,
        title: String,
        url: String,
    },
    Erased {
        id: &'a Id,
        url: String,
    },
    Unknown {
        id: &'a Id,
        url: String,
    },
    Image {
        title: String,
        document_url: String,
        blob_url: String,
    },
    Audio {
        document_url: String,
        blob_url: String,
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

        if document.is_erased() {
            return serde_json::to_value(RefMode::Erased {
                id: &document.id,
                url: document_url(&document.id, &None),
            })
            .context("failed to serialize");
        }

        if !self.preview_attachments || !Attachment::is_attachment(&document) {
            let title = arhiv.get_schema().get_title(&document)?;

            return serde_json::to_value(RefMode::Ref {
                document_type: &document.document_type,
                subtype: &document.subtype,
                title,
                url: document_url(&document.id, &None),
            })
            .context("failed to serialize");
        }

        let title = arhiv.get_schema().get_title(&document)?;

        let attachment: Attachment = document.try_into()?;

        if attachment.is_image() {
            return serde_json::to_value(RefMode::Image {
                title,
                document_url: document_url(&attachment.id, &None),
                blob_url: blob_url(&attachment.data.blob),
            })
            .context("failed to serialize");
        }

        if attachment.is_audio() {
            return serde_json::to_value(RefMode::Audio {
                document_url: document_url(&attachment.id, &None),
                blob_url: blob_url(&attachment.data.blob),
            })
            .context("failed to serialize");
        }

        serde_json::to_value(RefMode::Ref {
            document_type: &attachment.document_type,
            subtype: &attachment.subtype,
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
