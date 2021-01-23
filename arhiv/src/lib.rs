#![deny(clippy::all)]
#![deny(clippy::pedantic)]

mod arhiv;
mod config;
pub mod entities;
pub mod generator;
pub mod markup;
mod prime;
mod replica;
mod schema;
mod storage;

pub use crate::arhiv::test_arhiv::TestArhiv;
pub use crate::arhiv::{Arhiv, AttachmentLocation};
pub use config::Config;
pub use prime::server::start_server;
pub use schema::DocumentData;
pub use storage::{Filter, FilterMode, ListPage, Matcher, OrderBy};
