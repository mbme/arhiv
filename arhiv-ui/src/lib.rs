#![deny(clippy::all)]

use arhiv_core::entities::Id;

mod dto;
mod server;

pub use server::start_ui_server;

pub fn get_document_url(id: &Id, port: u16) -> String {
    format!("http://localhost:{}?id={}", port, id)
}
