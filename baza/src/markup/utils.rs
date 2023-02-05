use crate::entities::Id;

const REF_LINK_PREFIX: &str = "ref:";

#[must_use]
pub fn extract_id(text: &str) -> Option<Id> {
    if text.starts_with(REF_LINK_PREFIX) && text.len() > REF_LINK_PREFIX.len() {
        let id = text[REF_LINK_PREFIX.len()..].into();

        Some(id)
    } else {
        None
    }
}

#[must_use]
pub fn create_link(url: &str, text: &str) -> String {
    if text.is_empty() {
        format!("<{url}>")
    } else {
        format!("[{text}]({url})")
    }
}

#[must_use]
pub fn create_ref(id: &Id, text: &str) -> String {
    create_link(&format!("{REF_LINK_PREFIX}{id}"), text)
}
