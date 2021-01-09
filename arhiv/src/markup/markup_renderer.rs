use crate::{Arhiv, AttachmentLocation};
use pulldown_cmark::LinkType;
use pulldown_cmark::{html, Event, Tag};

use super::utils::extract_id;
use super::MarkupString;

pub struct MarkupRenderer<'a> {
    string: &'a MarkupString,
    arhiv: &'a Arhiv,
    document_path: String,
}

impl<'a> MarkupRenderer<'a> {
    pub fn new(string: &'a MarkupString, arhiv: &'a Arhiv, document_path: String) -> Self {
        MarkupRenderer {
            string,
            arhiv,
            document_path,
        }
    }

    pub fn to_html(&self) -> String {
        let parser = self.string.parse().map(|event| -> Event {
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
                                log::warn!(
                                    "Got broken reference: {} ({:?} {} {})",
                                    &id,
                                    link_type,
                                    destination,
                                    title
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
                            format!("{}/{}", &self.document_path, id),
                        );
                    }

                    let info = document
                        .get_attachment_info()
                        .expect("must be able to get attachment info");

                    let attachment_location = {
                        let attachment_location = self
                            .arhiv
                            .get_attachment_location(&id)
                            .expect("must be able to get attachment location");

                        match attachment_location {
                            AttachmentLocation::Url(location) => location,
                            AttachmentLocation::File(location) => format!("file://{}", location),
                        }
                    };

                    // render Image
                    if is_image_file(&info.filename) {
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
