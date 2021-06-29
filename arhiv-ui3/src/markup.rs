use arhiv_core::{
    entities::Attachment,
    markup::{extract_id, MarkupStr},
    pulldown_cmark::{html, Event, LinkType, Tag},
    Arhiv,
};
use rs_utils::log::warn;

pub trait MarkupStringExt {
    fn to_html(&self, arhiv: &Arhiv) -> String;
}

impl<'ms> MarkupStringExt for MarkupStr<'ms> {
    fn to_html(&self, arhiv: &Arhiv) -> String {
        let mut autolink_text: Option<String> = None;

        let parser = self.parse().map(|event| -> Event {
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

                    let document = arhiv
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
                        return link_event(normalized_title, format!("/documents/{}", id));
                    }

                    let attachment_location = format!("/attachment-data/{}", id);

                    let attachment =
                        Attachment::from(document).expect("document must be attachment");

                    // render Image
                    if attachment.is_image() {
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
