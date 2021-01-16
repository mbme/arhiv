#![deny(clippy::all)]
#![deny(clippy::pedantic)]

mod arhiv;
mod config;
pub mod entities;
pub mod generator;
pub mod markup;
mod schema;
mod storage;

pub use crate::arhiv::{start_server, Arhiv, AttachmentLocation};
pub use config::Config;
pub use schema::DocumentData;
pub use storage::{Filter, FilterMode, ListPage, Matcher, OrderBy};
