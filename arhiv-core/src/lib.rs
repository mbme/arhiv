#![deny(clippy::all)]
#![deny(clippy::pedantic)]

mod arhiv;
mod config;
pub mod entities;
pub mod markup;
pub mod prime_server;
pub mod schema;

#[cfg(test)]
mod tests;

pub use crate::arhiv::*;
pub use config::Config;
