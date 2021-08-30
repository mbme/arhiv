use crate::entities::Id;

const LINK_PREFIX: &str = "ref:";

#[must_use]
pub fn extract_id(text: &str) -> Option<Id> {
    if text.starts_with(LINK_PREFIX) && text.len() > LINK_PREFIX.len() {
        let id = text[LINK_PREFIX.len()..].into();

        Some(id)
    } else {
        None
    }
}

#[must_use]
pub fn create_link(url: &str, text: &str) -> String {
    if text.is_empty() {
        format!("<{}>", url)
    } else {
        format!("[{}]({})", text, url)
    }
}

#[must_use]
pub fn create_ref(id: &Id, text: &str) -> String {
    create_link(&format!("{}{}", LINK_PREFIX, id), text)
}
