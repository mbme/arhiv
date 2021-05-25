#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::{http::ContentType, response::Content};

use arhiv::Arhiv;
use catalog_index_page::*;
use catalog_page::*;
use document_editor_page::*;
use document_page::*;
use index_page::*;
use not_found_page::*;

use crate::utils::TemplateContext;

mod catalog_index_page;
mod catalog_page;
mod document_editor_page;
mod document_page;
mod index_page;
mod not_found_page;
mod utils;

fn main() {
    rocket::ignite()
        .manage(Arhiv::must_open())
        .manage(TemplateContext::new())
        .mount(
            "/",
            routes![
                render_favicon,              // /favicon.svg
                render_index_page,           // /
                render_catalog_index_page,   // /catalogs
                render_catalog_page,         // /catalogs/:document_type
                render_document_page,        // /documents/:id
                render_document_editor_page, // /documents/:id/edit
            ],
        )
        .register(catchers![render_not_found_page])
        .launch();
}

#[get("/favicon.svg")]
fn render_favicon() -> Content<&'static str> {
    Content(ContentType::SVG, include_str!("../public/favicon.svg"))
}
