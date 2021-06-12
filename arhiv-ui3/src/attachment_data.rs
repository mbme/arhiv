use anyhow::*;
use hyper::{header, Body, Request, Response};
use routerify::ext::RequestExt;
use rs_utils::read_file_as_stream;

use crate::{
    app_context::AppContext,
    http_utils::{not_found, AppResponse},
};
use arhiv_core::entities::BLOBHash;

pub async fn attachment_data_handler(req: Request<Body>) -> AppResponse {
    let hash = req.param("hash").unwrap();
    let hash = BLOBHash::from_string(hash);

    let context: &AppContext = req.data().unwrap();

    let attachment_data = context.arhiv.get_attachment_data(hash)?;

    if !attachment_data.exists()? {
        return not_found();
    }

    let file = read_file_as_stream(&attachment_data.path).await?;

    Response::builder()
        .header(
            // max caching
            header::CACHE_CONTROL,
            "immutable, private, max-age=31536000",
        )
        .body(Body::wrap_stream(file))
        .context("failed to build response")
}
