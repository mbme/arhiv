use anyhow::Result;
use askama::Template;

use arhiv::Arhiv;
use rocket::State;

#[derive(Template)]
#[template(path = "index_page.html")]
pub struct IndexPage {
    status: String,
}

#[get("/")]
pub fn render_index_page(arhiv: State<Arhiv>) -> Result<IndexPage> {
    let status = &arhiv.get_status()?;

    let page = IndexPage {
        status: status.to_string(),
    };

    Ok(page)
}
