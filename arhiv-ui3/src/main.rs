#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::{config::Environment, Config};
use rocket_contrib::{serve::StaticFiles, templates::Template};

use arhiv::markup::RenderOptions;
use pages::*;
use utils::{get_nav_document_types, AppContext};

mod components;
mod pages;
mod utils;

fn main() {
    let config = Config::build(Environment::Development)
        .extra("template_dir", "src/")
        .finalize()
        .expect("rocket config must be valid");

    rocket::custom(config)
        .mount(
            "/",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/public")),
        )
        .mount(
            "/",
            routes![
                index_page,           // /
                catalog_page,         // /catalogs/:document_type
                document_page,        // /documents/:id
                document_editor_page, // /documents/:id/edit
            ],
        )
        .manage(AppContext::new(RenderOptions {
            document_path: "/documents".to_string(),
            attachment_data_path: "/attachment-data".to_string(),
        }))
        .attach(Template::custom(|engines| {
            engines.tera.register_function(
                "get_nav_document_types",
                Box::new(|_args| Ok(get_nav_document_types().into())),
            );
        }))
        .register(catchers![not_found_page])
        .launch();
}
