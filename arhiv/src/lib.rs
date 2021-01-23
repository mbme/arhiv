#![deny(clippy::all)]
#![deny(clippy::pedantic)]

mod arhiv;
mod config;
mod data_service;
pub mod entities;
pub mod generator;
pub mod markup;
mod path_manager;
mod prime;
mod replica;
mod schema;
mod storage;

pub use crate::arhiv::test_arhiv::TestArhiv;
pub use crate::arhiv::Arhiv;
pub use config::Config;
pub use prime::server::start_server;
pub use schema::DocumentData;
pub use storage::{Filter, FilterMode, ListPage, Matcher, OrderBy};
