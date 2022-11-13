#![deny(clippy::all)]
#![allow(clippy::module_inception)]

mod baza;
mod db;
mod db_migrations;
pub mod entities;
pub mod markup;
mod path_manager;
pub mod schema;
mod sync;
pub mod validator;

#[cfg(test)]
mod tests;

pub use baza::Baza;
pub use db::*;
