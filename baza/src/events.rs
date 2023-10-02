use std::fmt::Display;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "typeName")]
pub enum BazaEvent {
    DocumentStaged {},
    DocumentsCommitted {},
}

impl Display for BazaEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            BazaEvent::DocumentStaged {} => "DocumentStaged",
            BazaEvent::DocumentsCommitted {} => "DocumentsCommitted",
        };

        write!(f, "{}", name)
    }
}
