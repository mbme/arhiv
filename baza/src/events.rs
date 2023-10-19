use std::fmt::Display;

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "typeName")]
pub enum BazaEvent {
    DocumentStaged {},
    DocumentsCommitted {},
    InstanceOutdated {},
    PeerDiscovered {},
    Synced {},
}

impl Display for BazaEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            BazaEvent::DocumentStaged {} => "DocumentStaged",
            BazaEvent::DocumentsCommitted {} => "DocumentsCommitted",
            BazaEvent::InstanceOutdated {} => "InstanceOutdated",
            BazaEvent::PeerDiscovered {} => "PeerDiscovered",
            BazaEvent::Synced {} => "Synced",
        };

        write!(f, "{}", name)
    }
}
