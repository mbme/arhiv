#![deny(clippy::all)]
#![deny(clippy::pedantic)]

mod arhiv;
mod client;
mod config;
pub mod entities;
pub mod generator;
pub mod markup;
mod schema;
mod server;
mod storage;

pub use crate::arhiv::test_arhiv::TestArhiv;
pub use crate::arhiv::{Arhiv, AttachmentLocation};
pub use config::Config;
pub use schema::DocumentData;
pub use server::start_server;
pub use storage::{Filter, FilterMode, ListPage, Matcher, OrderBy};
