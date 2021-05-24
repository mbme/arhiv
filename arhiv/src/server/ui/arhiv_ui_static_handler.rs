use warp::{path::Tail, Rejection, Reply};

use arhiv_ui::Assets;
use rs_utils::log;

pub async fn arhiv_ui_index_handler() -> Result<impl Reply, Rejection> {
    serve_impl("index.html")
}

pub async fn arhiv_ui_static_handler(path: Tail) -> Result<impl Reply, Rejection> {
    serve_impl(path.as_str())
}

fn serve_impl(path: &str) -> Result<impl Reply, Rejection> {
    log::debug!("GET {}", path);

    let asset = Assets::get(path).ok_or_else(warp::reject::not_found)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mut res = warp::reply::Response::new(asset.into());
    res.headers_mut().insert(
        "content-type",
        warp::http::header::HeaderValue::from_str(mime.as_ref()).unwrap(),
    );
    Ok(res)
}
