use anyhow::ensure;
use hyper::{http::request::Parts, Body, Request};
use routerify::ext::RequestExt;
use serde::Deserialize;
use serde_json::Value;

use crate::app_context::AppContext;
use arhiv_core::entities::{Document, Id};
use rs_utils::{
    run_command,
    server::{json_response, ServerResponse},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RPCAction {
    Delete { id: Id },
    Archive { id: Id, archive: bool },
    Save { document: Document },
    PickAttachment {},
}

pub async fn rpc_handler(req: Request<Body>) -> ServerResponse {
    let (parts, body): (Parts, Body) = req.into_parts();
    let body = hyper::body::to_bytes(body).await?;
    let action: RPCAction = serde_json::from_slice(&body)?;

    let context = parts.data::<AppContext>().unwrap();

    let mut response = Value::Null;

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
        RPCAction::PickAttachment {} => {
            let files = run_command("mb-filepicker", vec![])?;
            let files: Vec<String> = serde_json::from_str(&files)?;
            ensure!(files.len() < 2);

            if let Some(file_path) = files.get(0) {
                let document = context.arhiv.add_attachment(&file_path, false)?;
                response = Value::String(document.id.to_string());
            }
        }
    }

    json_response(response)
}
