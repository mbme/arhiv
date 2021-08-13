use anyhow::*;
use hyper::{http::request::Parts, Body, Request};
use routerify::ext::RequestExt;
use serde::Deserialize;
use serde_json::Value;

use arhiv_core::{
    entities::{Document, Id},
    Arhiv,
};
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

    let arhiv: &Arhiv = parts.data().unwrap();

    let mut response = Value::Null;

    match action {
        RPCAction::Delete { id } => {
            arhiv.delete_document(&id)?;
        }
        RPCAction::Archive { id, archive } => {
            arhiv.archive_document(&id, archive)?;
        }
        RPCAction::Save { mut document } => {
            let data_description = arhiv
                .get_schema()
                .get_data_description(&document.document_type)?;

            // prepare raw fields
            for field in &data_description.fields {
                let raw_value = {
                    if let Some(value) = document.data.get_str(field.name) {
                        value
                    } else {
                        continue;
                    }
                };

                let value = field
                    .from_string(raw_value)
                    .context("failed to extract value from string")?;

                document.data.set(field.name, value);
            }

            arhiv.stage_document(&mut document)?;
        }
        RPCAction::PickAttachment {} => {
            let files = run_command("mb-filepicker", vec![])?;
            let files: Vec<String> = serde_json::from_str(&files)?;
            ensure!(files.len() < 2);

            if let Some(file_path) = files.get(0) {
                let document = arhiv.add_attachment(&file_path, false)?;
                response = Value::String(document.id.to_string());
            }
        }
    }

    json_response(response)
}
