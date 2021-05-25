use anyhow::Result;
use askama::Template;

use arhiv::Arhiv;
use rocket::State;

use crate::utils::TemplateContext;

#[derive(Template)]
#[template(path = "index_page.html")]
pub struct IndexPage {
    context: TemplateContext,
    status: String,
}

#[get("/")]
pub fn render_index_page(
    arhiv: State<Arhiv>,
    context: State<TemplateContext>,
) -> Result<IndexPage> {
    let status = &arhiv.get_status()?;

    let page = IndexPage {
        context: context.clone(),
        status: status.to_string(),
    };

    Ok(page)
}
