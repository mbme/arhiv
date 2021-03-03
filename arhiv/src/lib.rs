#![deny(clippy::all)]
#![deny(clippy::pedantic)]

mod arhiv;
mod commander;
mod config;
pub mod entities;
pub mod markup;
pub mod schema;
pub mod server;

#[cfg(test)]
mod tests;

pub use crate::arhiv::*;
pub use config::Config;
