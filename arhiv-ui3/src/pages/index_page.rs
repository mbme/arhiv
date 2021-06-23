use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use crate::{components::Breadcrumbs, utils::render_page};
use arhiv_core::Arhiv;
use rs_utils::server::ServerResponse;

pub async fn index_page(req: Request<Body>) -> ServerResponse {
    let arhiv: &Arhiv = req.data().unwrap();

    let status = arhiv.get_status()?;
    let breadcrumbs = Breadcrumbs::Index.render()?;

    render_page(
        "pages/index_page.html.tera",
        json!({
            "breadcrumbs": breadcrumbs,
            "status": status.to_string(),
        }),
    )
}
