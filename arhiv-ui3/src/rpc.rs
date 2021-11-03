use anyhow::*;
use hyper::{http::request::Parts, Body, Request};
use routerify::ext::RequestExt;
use serde::Deserialize;
use serde_json::Value;

use arhiv_core::{entities::Id, Arhiv};
use rs_utils::{
    run_command,
    server::{json_response, ServerResponse},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RPCAction {
    Delete { id: Id },
    PickAttachment {},
}

pub async fn rpc_handler(req: Request<Body>) -> ServerResponse {
    let (parts, body): (Parts, Body) = req.into_parts();
    let body = hyper::body::to_bytes(body).await?;
    let action: RPCAction = serde_json::from_slice(&body)?;

    let arhiv: &Arhiv = parts.data().unwrap();

    let mut response = Value::Null;

    match action {
        RPCAction::Delete { id } => {
            arhiv.delete_document(&id)?;
        }

        RPCAction::PickAttachment {} => {
            let files = run_command("mb-filepicker", vec![])?;
            let files: Vec<String> = serde_json::from_str(&files)?;
            ensure!(files.len() < 2);

            if let Some(file_path) = files.get(0) {
                let document = arhiv.add_attachment(file_path, false)?;
                response = Value::String(document.id.to_string());
            }
        }
    }

    json_response(response)
}
