#![deny(clippy::all)]
#![deny(clippy::pedantic)]

mod arhiv;
mod config;
pub mod entities;
pub mod markup;
pub mod schema;
mod server;

#[cfg(test)]
mod tests;

pub use crate::arhiv::*;
pub use config::Config;
pub use server::{get_attachment_data_handler, start_server};
