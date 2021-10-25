use anyhow::*;
use hyper::{http::request::Parts, Body, Request};
use routerify::ext::RequestExt;
use serde::Deserialize;
use serde_json::Value;

use arhiv_core::{
    entities::{Document, Id},
    Arhiv, Filter,
};
use rs_utils::{
    run_command,
    server::{json_response, ServerResponse},
};

use crate::components::Catalog;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RPCAction {
    Delete {
        id: Id,
    },
    Save {
        document: Box<Document>,
    },
    PickAttachment {},
    RenderCatalog {
        picker_mode: bool,
        filter: Filter,
    },
    SearchCatalog {
        parent_collection: Option<Id>,
        picker_mode: bool,
        document_type: Option<String>,
        pattern: String,
    },
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

        RPCAction::Save { mut document } => {
            let data_description = arhiv
                .get_schema()
                .get_data_description(&document.document_type)?;

            // prepare raw fields
            for field in &data_description.fields {
                let raw_value = document.data.get_str(field.name).unwrap_or_default();

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
                let document = arhiv.add_attachment(file_path, false)?;
                response = Value::String(document.id.to_string());
            }
        }

        RPCAction::RenderCatalog {
            filter,
            picker_mode,
        } => {
            let mut catalog = Catalog::from_filter(filter);

            if picker_mode {
                catalog = catalog.picker_mode();
            }

            let catalog = catalog.render(arhiv)?;

            response = Value::String(catalog);
        }

        RPCAction::SearchCatalog {
            parent_collection,
            picker_mode,
            document_type,
            pattern,
        } => {
            let mut catalog = Catalog::new().search(pattern);

            if let Some(parent_collection) = parent_collection {
                catalog = catalog.in_collection(parent_collection);
            }

            if let Some(document_type) = document_type {
                catalog = catalog.with_type(document_type);
            }

            if picker_mode {
                catalog = catalog.picker_mode();
            }

            let catalog = catalog.render(arhiv)?;

            response = Value::String(catalog);
        }
    }

    json_response(response)
}
