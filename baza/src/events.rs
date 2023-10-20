use std::fmt::Display;

use serde::Serialize;

use crate::entities::Id;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "typeName")]
pub enum BazaEvent {
    DocumentStaged {},
    DocumentsCommitted {},
    InstanceOutdated {},
    PeerDiscovered {},
    Synced {},
    DocumentLocked { id: Id, reason: String },
    DocumentUnlocked { id: Id },
}

impl Display for BazaEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BazaEvent::DocumentStaged {} => write!(f, "DocumentStaged"),
            BazaEvent::DocumentsCommitted {} => write!(f, "DocumentsCommitted"),
            BazaEvent::InstanceOutdated {} => write!(f, "InstanceOutdated"),
            BazaEvent::PeerDiscovered {} => write!(f, "PeerDiscovered"),
            BazaEvent::Synced {} => write!(f, "Synced"),
            BazaEvent::DocumentLocked { id, reason } => write!(f, "DocumentLocked {id}: {reason}"),
            BazaEvent::DocumentUnlocked { id } => write!(f, "DocumentUnlocked {id}"),
        }
    }
}
