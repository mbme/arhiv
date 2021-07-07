use arhiv_core::{
    markup::{extract_id, MarkupStr},
    pulldown_cmark::{html, Event, Tag},
    Arhiv,
};

use crate::components::Ref;

pub trait MarkupStringExt {
    fn to_html(&self, arhiv: &Arhiv) -> String;
}

impl<'ms> MarkupStringExt for MarkupStr<'ms> {
    fn to_html(&self, arhiv: &Arhiv) -> String {
        let mut matched = false;

        let parser = self.parse().map(|event| -> Event {
            // FIXME handle images
            match event {
                Event::Text(_) if matched => {
                    // ignore text inside link
                    return Event::Text("".into());
                }
                Event::End(Tag::Link(_, _, _)) if matched => {
                    matched = false;
                    return Event::Text("".into());
                }
                Event::Start(Tag::Link(ref _link_type, ref destination, ref _title)) => {
                    let id = match extract_id(destination) {
                        Some(id) => id,
                        None => {
                            return event;
                        }
                    };

                    matched = true;

                    let link = Ref::from_id(id)
                        .preview_attachments()
                        .render(arhiv)
                        .expect("failed to render ref");

                    return Event::Html(link.into());
                }
                _ => event,
            }
        });

        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        html_output
    }
}
