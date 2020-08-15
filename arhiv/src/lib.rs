#![deny(clippy::all)]
#![deny(clippy::pedantic)]

mod arhiv;
pub mod config;
pub mod entities;
mod fs_transaction;
mod storage;
pub mod utils;

pub use crate::arhiv::Arhiv;
pub use storage::{Matcher, QueryFilter};
