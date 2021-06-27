use anyhow::*;
use pulldown_cmark::{html, Event, LinkType, Tag};
use serde::Serialize;

use crate::schema::{FieldType, SCHEMA};
use crate::{entities::*, Arhiv};
use rs_utils::log::warn;

use super::utils::extract_id;
use super::MarkupStr;

pub struct MarkupRenderer<'a> {
    arhiv: &'a Arhiv,
    options: RenderOptions,
}

#[derive(Serialize, Clone)]
pub struct RenderOptions {
    pub document_path: String,
    pub attachment_data_path: String,
}

impl<'a> MarkupRenderer<'a> {
    pub fn new(arhiv: &'a Arhiv, options: RenderOptions) -> Self {
        MarkupRenderer { arhiv, options }
    }

    pub fn to_html(&self, markup: &MarkupStr) -> String {
        let mut autolink_text: Option<String> = None;

        let parser = markup.parse().map(|event| -> Event {
            // FIXME handle images
            match event {
                // replace default autolink text with normalized title
                Event::Text(_) if autolink_text.is_some() => {
                    return Event::Text(autolink_text.take().unwrap().into());
                }
                Event::Start(Tag::Link(ref link_type, ref destination, ref title)) => {
                    let id = match extract_id(destination) {
                        Some(id) => id,
                        None => {
                            return event;
                        }
                    };

                    let normalized_title: String = if title.is_empty() {
                        id.to_string()
                    } else {
                        title.to_string()
                    };

                    // if autolink, save normalized title to use it in the text node
                    match link_type {
                        LinkType::Autolink => {
                            autolink_text = Some(normalized_title.clone());
                        }
                        _ => {}
                    };

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

                    if !Attachment::is_attachment(&document) {
                        // FIXME extract title
                        // render Document Link
                        return link_event(
                            normalized_title,
                            format!("{}/{}", &self.options.document_path, id),
                        );
                    }

                    let attachment_location = {
                        let attachment_data = self
                            .arhiv
                            .get_attachment_data(&id)
                            .expect("must be able to get attachment data");

                        if attachment_data.exists().unwrap() {
                            format!("{}/{}", &self.options.attachment_data_path, &id)
                        } else if self
                            .arhiv
                            .is_prime()
                            .expect("must be able to check prime status")
                        {
                            // this is a prime instance, data is missing, so just render Document Link
                            return link_event(
                                normalized_title,
                                format!("{}/{}", &self.options.document_path, id),
                            );
                        } else {
                            self.arhiv
                                .get_network_service()
                                .unwrap()
                                .get_attachment_data_url(&id)
                        }
                    };

                    let attachment =
                        Attachment::from(document).expect("document must be attachment");
                    let filename = attachment.get_filename();

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

    pub fn get_preview(&self, document: &Document) -> Result<String> {
        let field = SCHEMA.pick_title_field(&document.document_type)?;

        match field.field_type {
            FieldType::MarkupString {} => {
                let text = document.get_field_str(field.name).unwrap_or_default();
                let preview = text.lines().take(4).collect::<Vec<_>>().join("\n");
                let markup: MarkupStr = preview.as_str().into();

                Ok(self.to_html(&markup))
            }

            FieldType::String {} => {
                let value = document.get_field_str(field.name).unwrap_or_default();

                Ok(value.to_string())
            }
            _ => unimplemented!(),
        }
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
