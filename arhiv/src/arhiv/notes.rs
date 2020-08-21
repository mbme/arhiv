use super::{Arhiv, DocumentFilter, Matcher};
use crate::entities::*;
use serde_json::{json, Value};

pub struct ArhivNotes {
    pub arhiv: Arhiv,
}

impl Arhiv {
    pub fn notes(self) -> ArhivNotes {
        ArhivNotes::new(self)
    }
}

pub const NOTE_TYPE: &str = "note";

impl ArhivNotes {
    pub fn new(arhiv: Arhiv) -> ArhivNotes {
        ArhivNotes { arhiv }
    }

    pub fn create_note() -> Document {
        let mut document = Document::new(NOTE_TYPE);
        document.data = ArhivNotes::data("", "");

        document
    }

    pub fn data<S: Into<String>>(name: S, data: S) -> Value {
        json!({ "name": name.into(), "data": data.into() })
    }

    pub fn list(&self, pattern: String) -> Vec<Document> {
        let matcher = {
            if pattern.is_empty() {
                None
            } else {
                Some(Matcher {
                    selector: "$.name".to_string(),
                    pattern,
                })
            }
        };

        let filter = DocumentFilter {
            document_type: Some(NOTE_TYPE.to_string()),
            page_offset: None,
            page_size: None,
            matcher,
            skip_archived: Some(true),
        };

        self.arhiv
            .list_documents(Some(filter))
            .expect("must be able to list notes")
    }

    pub fn get_note(&self, id: &Id) -> Option<Document> {
        let result = self
            .arhiv
            .get_document(id)
            .expect("must be able to get note");

        if let Some(ref document) = result {
            assert_eq!(document.document_type, NOTE_TYPE);
        }

        result
    }

    pub fn put_note(&self, note: Document) {
        self.arhiv
            .stage_document(note)
            .expect("must be able to save note");
    }
}
