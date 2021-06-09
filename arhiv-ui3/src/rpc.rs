use anyhow::*;
use rocket::{http::Status, State};
use rocket_contrib::json::Json;
use serde::Deserialize;

use crate::app_context::AppContext;
use arhiv::entities::{Document, Id};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RPCAction {
    Delete { id: Id },
    Archive { id: Id, archive: bool },
    Save { document: Document },
}

#[post("/rpc", format = "json", data = "<action>")]
pub fn rpc_endpoint(action: Json<RPCAction>, context: State<AppContext>) -> Result<Status> {
    match action.into_inner() {
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

    Ok(Status::Ok)
}
