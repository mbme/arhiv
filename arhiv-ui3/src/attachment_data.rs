use hyper::{Body, Request};
use routerify::ext::RequestExt;

use crate::app_context::AppContext;
use arhiv_core::{entities::BLOBHash, prime_server::respond_with_attachment_data};
use rs_utils::server::ServerResponse;

pub async fn attachment_data_handler(req: Request<Body>) -> ServerResponse {
    let hash = req.param("hash").unwrap();
    let hash = BLOBHash::from_string(hash);

    let context: &AppContext = req.data().unwrap();

    respond_with_attachment_data(&context.arhiv, hash).await
}
