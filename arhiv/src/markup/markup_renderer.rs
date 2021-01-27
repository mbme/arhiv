use crate::{
    entities::{AttachmentLocation, AttachmentSource},
    Arhiv,
};
use pulldown_cmark::LinkType;
use pulldown_cmark::{html, Event, Tag};
use rs_utils::log::warn;
use serde::{Deserialize, Serialize};

use super::utils::extract_id;
use super::MarkupString;

pub struct MarkupRenderer<'a> {
    arhiv: &'a Arhiv,
    options: &'a RenderOptions,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RenderOptions {
    document_path: String,
    new_attachments: Vec<AttachmentSource>,
}

impl<'a> MarkupRenderer<'a> {
    pub fn new(arhiv: &'a Arhiv, options: &'a RenderOptions) -> Self {
        MarkupRenderer { arhiv, options }
    }

    pub fn to_html(&self, string: &'a MarkupString) -> String {
        let parser = string.parse().map(|event| -> Event {
            match event {
                // FIXME handle images
                Event::Start(Tag::Link(ref link_type, ref destination, ref title)) => {
                    let id = match extract_id(destination) {
                        Some(id) => id,
                        None => {
                            return event;
                        }
                    };

                    let normalized_title: String = if title.is_empty() {
                        id.to_string().into()
                    } else {
                        title.to_string()
                    };

                    // render new attachment
                    if let Some(new_attachment) = self
                        .options
                        .new_attachments
                        .iter()
                        .find(|item| item.id == id)
                    {
                        // render Image
                        if is_image_file(&new_attachment.filename) {
                            return Event::Start(Tag::Image(
                                LinkType::Inline,
                                new_attachment.file_path.clone().into(),
                                normalized_title.into(),
                            ));
                        }

                        // render Attachment Link
                        return link_event(new_attachment.file_path.clone(), normalized_title);
                    }

                    let document = self
                        .arhiv
                        .get_document(&id)
                        .expect("must be able to get document");

                    let document = {
                        match document {
                            Some(document) => document,
                            None => {
                                warn!(
                                    "Got broken reference: {} ({:?} {} {})",
                                    &id, link_type, destination, title
                                );

                                return event;
                            }
                        }
                    };

                    if !document.is_attachment() {
                        // FIXME extract title
                        // render Document Link
                        return link_event(
                            normalized_title,
                            format!("{}/{}", &self.options.document_path, id),
                        );
                    }

                    let attachment_location = {
                        let attachment_location = self
                            .arhiv
                            .get_attachment_location(&id)
                            .expect("must be able to get attachment location");

                        match attachment_location {
                            AttachmentLocation::Url(location) => location,
                            AttachmentLocation::File(location) => format!("file:{}", location),
                        }
                    };

                    let filename = self
                        .arhiv
                        .schema
                        .get_field_string(&document, "filename")
                        .expect("must be able to read filename");

                    // render Image
                    if is_image_file(&filename) {
                        return Event::Start(Tag::Image(
                            LinkType::Inline,
                            attachment_location.into(),
                            normalized_title.into(),
                        ));
                    }

                    // render Attachment Link
                    return link_event(attachment_location, normalized_title);
                }
                _ => event,
            }
        });

        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        html_output
    }
}

fn link_event<'a>(title: String, destination: String) -> Event<'a> {
    Event::Start(Tag::Link(
        LinkType::Inline,
        destination.into(),
        title.into(),
    ))
}

fn is_image_file(filename: &str) -> bool {
    let filename = filename.to_lowercase();

    return filename.ends_with(".png")
        || filename.ends_with(".jpg")
        || filename.ends_with(".jpeg")
        || filename.ends_with(".svg");
}
