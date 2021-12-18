use hyper::{Body, Request};
use routerify::ext::RequestExt;

use arhiv_core::{entities::BLOBId, prime_server::respond_with_blob, Arhiv};
use rs_utils::server::ServerResponse;

pub async fn blob_handler(req: Request<Body>) -> ServerResponse {
    let blob_id = req.param("blob_id").unwrap().as_str();
    let blob_id = BLOBId::from_string(blob_id);

    let arhiv: &Arhiv = req.data().unwrap();

    respond_with_blob(arhiv, &blob_id).await
}
