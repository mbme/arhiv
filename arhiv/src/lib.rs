#![deny(clippy::all)]
#![deny(clippy::pedantic)]

mod arhiv;
mod config;
pub mod entities;
pub mod generator;
pub mod markup;
pub mod modules;
mod storage;

pub use crate::arhiv::{start_server, Arhiv, AttachmentLocation};
pub use config::Config;
pub use storage::{
    AttachmentFilter, DocumentFilter, DocumentFilterMode, ListPage, Matcher, OrderBy,
};
