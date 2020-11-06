use crate::extract_refs;
use crate::DocumentImpl;
use crate::MarkupString;
use arhiv::entities::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Default)]
pub struct NoteData {
    pub name: String,
    pub data: MarkupString,
}

pub struct Note(pub Document<NoteData>);

impl Note {
    pub fn new() -> Self {
        Note(Document::new(Self::TYPE, NoteData::default()))
    }
}

impl DocumentImpl for Note {
    const TYPE: &'static str = "note";

    type Data = NoteData;

    fn from_document(document: Document) -> Self {
        assert_eq!(document.document_type, Self::TYPE, "Not a note");

        Note(document.into())
    }

    fn into_document(self) -> Document<Self::Data> {
        self.0
    }

    fn extract_refs(&self) -> HashSet<Id> {
        let nodes = self.0.data.data.parse();

        extract_refs(&nodes)
    }
}
