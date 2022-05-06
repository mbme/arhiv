#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::module_inception,
    clippy::wildcard_imports,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless
)]

mod arhiv;
mod config;
pub mod definitions;
pub mod entities;
pub mod markup;
mod path_manager;
pub mod prime_server;
pub mod schema;
pub mod test_arhiv;
mod validator;

#[cfg(test)]
mod tests;

pub use crate::arhiv::db::{
    ArhivConnection, BLOBSCount, Conditions, DocumentsCount, Filter, ListPage, OrderBy,
};
pub use crate::arhiv::Arhiv;
pub use config::Config;
pub use validator::{FieldValidationErrors, ValidationError, Validator};

pub use pulldown_cmark;
