#![deny(clippy::all)]
#![deny(clippy::pedantic)]

mod arhiv;
mod config;
pub mod entities;
mod fs_transaction;
mod storage;
pub mod utils;

pub use crate::arhiv::notes::ArhivNotes;
pub use crate::arhiv::Arhiv;
pub use config::Config;
pub use storage::{Matcher, QueryFilter};
