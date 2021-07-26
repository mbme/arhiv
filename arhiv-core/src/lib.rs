#![deny(clippy::all)]
#![deny(clippy::pedantic)]

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

pub use definitions::get_schema;
pub use pulldown_cmark;
