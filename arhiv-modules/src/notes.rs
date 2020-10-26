use crate::DocumentImpl;
use arhiv::entities::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct NoteData {
    pub name: String,
    pub data: String,
}

pub struct Note(pub Document<NoteData>);

impl DocumentImpl for Note {
    const TYPE: &'static str = "note";

    type Data = NoteData;

    fn new() -> Self {
        Note(Document::new(Self::TYPE))
    }

    fn from(document: Document) -> Self {
        assert_eq!(document.document_type, Self::TYPE, "Not a note");

        Note(document.into())
    }

    fn into_document(self) -> Document<Self::Data> {
        self.0
    }
}
