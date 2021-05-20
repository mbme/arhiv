use pulldown_cmark::LinkType;
use pulldown_cmark::{html, Event, Tag};

use crate::{entities::*, Arhiv};
use rs_utils::log::warn;

use super::utils::extract_id;
use super::MarkupString;

pub struct MarkupRenderer<'a> {
    arhiv: &'a Arhiv,
    options: &'a RenderOptions,
}

pub struct RenderOptions {
    pub document_path: String,
    pub attachment_data_path: String,
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
                            .get_attachment_data_by_id(&id)
                            .expect("must be able to get attachment data");

                        if attachment_data.exists().unwrap() {
                            format!(
                                "{}/{}",
                                &self.options.attachment_data_path, &attachment_data.hash
                            )
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
                                .get_attachment_data_url(&attachment_data.hash)
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
