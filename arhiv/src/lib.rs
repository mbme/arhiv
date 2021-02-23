#![deny(clippy::all)]
#![deny(clippy::pedantic)]

mod arhiv;
mod config;
pub mod entities;
pub mod markup;
pub mod schema;
mod server;

pub use crate::arhiv::*;
pub use config::Config;
pub use server::start_server;
