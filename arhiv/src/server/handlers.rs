use std::sync::Arc;

use anyhow::*;
use warp::{http, hyper, reply, Reply};

use crate::entities::*;
use crate::Arhiv;
use rs_utils::log;
use rs_utils::read_file_as_stream;

pub async fn get_attachment_data_handler(
    hash: String,
    arhiv: Arc<Arhiv>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let hash = BLOBHash::from_string(hash);
    log::debug!("Serving attachment data {}", &hash);

    let attachment_data = arhiv.get_attachment_data(hash).unwrap();

    if !attachment_data
        .exists()
        .expect("failed to check if attachment data exists")
    {
        log::warn!(
            "Requested attachment data {} is not found",
            &attachment_data.hash
        );

        return Ok(reply::with_status(
            format!("can't find attachment data {}", &attachment_data.hash),
            http::StatusCode::NOT_FOUND,
        )
        .into_response());
    }

    let file = match read_file_as_stream(&attachment_data.path).await {
        Ok(file) => file,
        Err(err) => {
            log::error!(
                "Failed to read attachment data {}: {}",
                &attachment_data.hash,
                &err
            );

            return Ok(reply::with_status(
                format!(
                    "failed to read attachment data {}: {}",
                    &attachment_data.hash, err
                ),
                http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response());
        }
    };

    // FIXME support ranges, status code: partial content
    Ok(http::Response::builder()
        .header("Cache-Control", "immutable, private, max-age=31536000") // max caching
        .body(hyper::Body::wrap_stream(file))
        .expect("must be able to construct response"))
}
