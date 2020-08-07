#![deny(clippy::all)]
#![deny(clippy::pedantic)]

mod arhiv;
pub mod config;
pub mod entities;
pub mod server;
pub mod utils;

pub use arhiv::storage::{Matcher, QueryFilter};
pub use arhiv::Arhiv;
