mod auto_commit_service;
mod backup;
pub mod baza2;
mod document_expert;
pub mod entities;
pub mod markup;
pub mod schema;
mod search;

#[cfg(test)]
mod tests;

pub use auto_commit_service::{AutoCommitService, AutoCommitTask};
pub use document_expert::DocumentExpert;

pub const DEV_MODE: bool = cfg!(not(feature = "production-mode"));
