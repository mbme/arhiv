#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket_contrib::{serve::StaticFiles, templates::Template};

use arhiv::Arhiv;
use catalog_index_page::*;
use catalog_page::*;
use document_editor_page::*;
use document_page::*;
use index_page::*;
use not_found_page::*;
use utils::{get_nav_document_types, TemplateContext};

mod catalog_index_page;
mod catalog_page;
mod document_editor_page;
mod document_page;
mod index_page;
mod not_found_page;
mod utils;

fn main() {
    rocket::ignite()
        .attach(Template::custom(|engines| {
            engines.tera.register_function(
                "get_nav_document_types",
                Box::new(|_args| Ok(get_nav_document_types().into())),
            );
        }))
        .manage(Arhiv::must_open())
        .manage(TemplateContext::new())
        .mount(
            "/",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/public")),
        )
        .mount(
            "/",
            routes![
                index_page,           // /
                catalog_index_page,   // /catalogs
                catalog_page,         // /catalogs/:document_type
                document_page,        // /documents/:id
                document_editor_page, // /documents/:id/edit
            ],
        )
        .register(catchers![not_found_page])
        .launch();
}
