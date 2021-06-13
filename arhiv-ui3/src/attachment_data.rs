use hyper::{Body, Request};
use routerify::ext::RequestExt;

use arhiv_core::{entities::BLOBHash, prime_server::respond_with_attachment_data, Arhiv};
use rs_utils::server::ServerResponse;

pub async fn attachment_data_handler(req: Request<Body>) -> ServerResponse {
    let hash = req.param("hash").unwrap();
    let hash = BLOBHash::from_string(hash);

    let arhiv: &Arhiv = req.data().unwrap();

    respond_with_attachment_data(arhiv, hash).await
}
