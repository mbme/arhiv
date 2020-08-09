#![deny(clippy::all)]
#![deny(clippy::pedantic)]

mod arhiv;
pub mod config;
pub mod entities;
mod storage;
pub mod utils;

pub use arhiv::Arhiv;
pub use storage::{Matcher, QueryFilter};
