use crate::DocumentImpl;
use crate::MarkupString;
use arhiv::entities::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum TaskComplexity {
    Unknown,
    Small,
    Medium,
    Large,
    Epic,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum TaskStatus {
    Inbox,
    Todo,
    Later,
    InProgress,
    Paused,
    Done,
    Cancelled,
}

#[derive(Serialize, Deserialize)]
pub struct TaskData {
    pub title: String,
    pub description: MarkupString,
    pub complexity: TaskComplexity,
    pub status: TaskStatus,
    pub project_id: Id,
}

pub struct Task(pub Document<TaskData>);

impl Task {
    pub fn new(project_id: Id) -> Self {
        let data = TaskData {
            title: "".to_owned(),
            description: "".to_owned().into(),
            complexity: TaskComplexity::Unknown,
            status: TaskStatus::Inbox,
            project_id,
        };

        Task(Document::new(Self::TYPE, data))
    }
}

impl DocumentImpl for Task {
    const TYPE: &'static str = "Task";

    type Data = TaskData;

    fn from_document(document: Document) -> Self {
        assert_eq!(document.document_type, Self::TYPE, "Not a task");

        Task(document.into())
    }

    fn into_document(self) -> Document<Self::Data> {
        self.0
    }

    fn extract_refs(&self) -> HashSet<Id> {
        let mut refs = self.0.data.description.extract_refs();

        refs.insert(self.0.data.project_id.clone());

        refs
    }
}
