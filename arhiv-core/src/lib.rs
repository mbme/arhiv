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
mod definitions;
pub mod entities;
pub mod markup;
pub mod prime_server;
pub mod schema;

#[cfg(test)]
mod tests;

pub use crate::arhiv::*;
pub use config::Config;

pub use definitions::get_standard_schema;
pub use pulldown_cmark;
