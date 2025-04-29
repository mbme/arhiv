mod auto_commit_service;
mod backup;
mod baza;
mod baza_info;
mod baza_manager;
mod baza_paths;
mod baza_state;
mod baza_storage;
mod document_expert;
pub mod entities;
pub mod markup;
mod merge_expert;
pub mod schema;

pub use baza::{BLOBSCount, Baza, DocumentsCount, StagingError, ValidationError};
pub use baza_info::BazaInfo;
pub use baza_manager::BazaManager;
pub use baza_paths::BazaPaths;
pub use baza_state::{BazaState, DocumentHead, Filter, ListPage, Locks};
pub use baza_storage::BazaStorage;

pub use auto_commit_service::{AutoCommitService, AutoCommitTask};
pub use document_expert::DocumentExpert;

pub const DEV_MODE: bool = cfg!(not(feature = "production-mode"));
