#![feature(proc_macro_hygiene, decl_macro)]

use arhiv::Arhiv;

use index_page::*;
use rocket::{http::ContentType, response::Content};

#[macro_use]
extern crate rocket;

mod index_page;

#[get("/favicon.svg")]
fn public_files() -> Content<&'static str> {
    Content(ContentType::SVG, include_str!("../public/favicon.svg"))
}

fn main() {
    let arhiv = Arhiv::must_open();

    rocket::ignite()
        .manage(arhiv)
        .mount("/", routes![public_files, render_index_page,])
        .launch();
}
