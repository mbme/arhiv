#![deny(clippy::all)]
#![allow(clippy::module_inception)]

mod arhiv;
mod config;
pub mod definitions;
pub mod entities;
pub mod markup;
mod path_manager;
pub mod prime_server;
pub mod schema;
mod scraper;
pub mod test_arhiv;
#[cfg(test)]
mod tests;
mod validator;

pub use crate::arhiv::db::{
    ArhivConnection, BLOBSCount, Conditions, DocumentsCount, Filter, ListPage, OrderBy,
};
pub use crate::arhiv::Arhiv;
pub use crate::scraper::ScraperOptions;
pub use config::Config;
pub use validator::{FieldValidationErrors, ValidationError, Validator};

pub use pulldown_cmark;
