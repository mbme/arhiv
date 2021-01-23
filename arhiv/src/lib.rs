#![deny(clippy::all)]
#![deny(clippy::pedantic)]

mod arhiv;
mod config;
mod data_service;
mod db;
pub mod entities;
pub mod markup;
mod path_manager;
mod prime;
mod replica;
pub mod schema;

pub use crate::arhiv::test_arhiv::TestArhiv;
pub use crate::arhiv::Arhiv;
pub use config::Config;
pub use db::{Filter, FilterMode, ListPage, Matcher, OrderBy};
pub use prime::server::start_server;
pub use schema::DocumentData;
