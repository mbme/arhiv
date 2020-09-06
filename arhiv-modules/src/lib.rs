#![deny(clippy::all)]
#![deny(clippy::pedantic)]

mod notes;

pub use notes::ArhivNotes;

pub fn create_link<S: Into<String>>(url: S, text: S) -> String {
    let url = url.into();
    let text = text.into();

    if text.is_empty() {
        format!("[[{}]]", url)
    } else {
        format!("[[{}][{}]]", url, text)
    }
}
