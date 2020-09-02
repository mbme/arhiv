#![deny(clippy::all)]
#![deny(clippy::pedantic)]

mod arhiv;
mod config;
pub mod entities;
mod storage;

pub use crate::arhiv::{start_server, Arhiv};
pub use config::Config;
pub use storage::{AttachmentFilter, DocumentFilter, Matcher};
