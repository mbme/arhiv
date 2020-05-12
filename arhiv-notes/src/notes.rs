use arhiv::entities::*;
use arhiv::{Arhiv, QueryFilter};

pub struct ArhivNotes {
    arhiv: Arhiv,
}

const NOTE_TYPE: &str = "note";

impl ArhivNotes {
    pub fn must_open() -> ArhivNotes {
        ArhivNotes {
            arhiv: Arhiv::must_open(),
        }
    }

    pub fn list(&self) -> Vec<Document> {
        let query = QueryFilter {
            document_type: Some(NOTE_TYPE.to_string()),
            page: None,
        };

        self.arhiv
            .list_documents(Some(query))
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
