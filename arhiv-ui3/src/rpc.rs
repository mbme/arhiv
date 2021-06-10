use hyper::{http::request::Parts, Body, Request, Response};
use routerify::ext::RequestExt;
use serde::Deserialize;

use crate::{app_context::AppContext, http_utils::AppResponse};
use arhiv::entities::{Document, Id};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RPCAction {
    Delete { id: Id },
    Archive { id: Id, archive: bool },
    Save { document: Document },
}

pub async fn rpc_handler(req: Request<Body>) -> AppResponse {
    let (parts, body): (Parts, Body) = req.into_parts();
    let body = hyper::body::to_bytes(body).await?;
    let action: RPCAction = serde_json::from_slice(&body)?;

    let context = parts.data::<AppContext>().unwrap();

    match action {
        RPCAction::Delete { id } => {
            context.arhiv.delete_document(&id)?;
        }
        RPCAction::Archive { id, archive } => {
            context.arhiv.archive_document(&id, archive)?;
        }
        RPCAction::Save { document } => {
            context.arhiv.stage_document(document)?;
        }
    }

    Ok(Response::new(Body::empty()))
}
