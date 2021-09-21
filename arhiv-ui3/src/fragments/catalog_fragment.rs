use hyper::{http::request::Parts, Body, Request};
use routerify::ext::RequestExt;

use arhiv_core::{Arhiv, Filter};
use rs_utils::server::{RequestQueryExt, ServerResponse};

use crate::{components::Catalog, utils::render_content};

pub async fn catalog_fragment(req: Request<Body>) -> ServerResponse {
    let parent_collection = req
        .get_query_param("parent_collection")
        .map(|value| value.into());

    let (parts, body): (Parts, Body) = req.into_parts();
    let body = hyper::body::to_bytes(body).await?;
    let filter: Filter = serde_json::from_slice(&body)?;

    let arhiv: &Arhiv = parts.data().unwrap();

    let catalog = Catalog::from_filter(filter)
        .in_collection(parent_collection)
        .render(arhiv)?;

    render_content(catalog)
}
