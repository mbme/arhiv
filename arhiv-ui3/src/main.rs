#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::{config::Environment, Config};

use app_context::AppContext;
use arhiv::markup::RenderOptions;
use pages::*;
use public_assets::*;

mod app_context;
mod components;
mod pages;
mod public_assets;

fn main() {
    let config = Config::build(Environment::Development)
        .extra("template_dir", "src/")
        .finalize()
        .expect("rocket config must be valid");

    rocket::custom(config)
        .mount(
            "/",
            routes![
                public_assets,        // /public
                index_page,           // /
                catalog_page,         // /catalogs/:document_type
                document_page,        // /documents/:id
                document_editor_page, // /documents/:id/edit
            ],
        )
        .manage(
            AppContext::new(RenderOptions {
                document_path: "/documents".to_string(),
                attachment_data_path: "/attachment-data".to_string(),
            })
            .expect("AppContext must init"),
        )
        .register(catchers![not_found_page])
        .launch();
}
