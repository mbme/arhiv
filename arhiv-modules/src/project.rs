use crate::extract_refs;
use crate::DocumentImpl;
use crate::MarkupString;
use arhiv::entities::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Default)]
pub struct ProjectData {
    pub title: String,
    pub description: MarkupString,
}

pub struct Project(pub Document<ProjectData>);

impl Project {
    pub fn new() -> Self {
        Project(Document::new(Self::TYPE, ProjectData::default()))
    }
}

impl DocumentImpl for Project {
    const TYPE: &'static str = "project";

    type Data = ProjectData;

    fn from_document(document: Document) -> Self {
        assert_eq!(document.document_type, Self::TYPE, "Not a project");

        Project(document.into())
    }

    fn into_document(self) -> Document<Self::Data> {
        self.0
    }

    fn extract_refs(&self) -> HashSet<Id> {
        let nodes = self.0.data.description.parse();

        extract_refs(&nodes)
    }
}
