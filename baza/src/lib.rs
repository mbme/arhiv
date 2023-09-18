#![deny(clippy::all)]
#![allow(clippy::module_inception)]

mod backup;
mod baza;
mod db;
mod db_migrations;
mod document_expert;
pub mod entities;
pub mod markup;
mod path_manager;
pub mod schema;
mod search;
pub mod sync;
pub mod validator;

#[cfg(test)]
mod tests;

pub use baza::Baza;
pub use db::*;
pub use document_expert::DocumentExpert;
