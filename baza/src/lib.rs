mod auto_commit_service;
mod backup;
mod baza;
mod db;
mod document_expert;
pub mod entities;
mod events;
pub mod markup;
mod path_manager;
pub mod schema;
mod search;
pub mod sync;
pub mod validator;

#[cfg(test)]
mod tests;

pub use auto_commit_service::{AutoCommitService, AutoCommitTask};
pub use baza::{Baza, BazaEvent, BazaOptions, Credentials};
pub use db::*;
pub use document_expert::DocumentExpert;

pub const DEV_MODE: bool = cfg!(not(feature = "production-mode"));
