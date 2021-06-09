use anyhow::*;
use rocket::{http::Status, State};
use rocket_contrib::json::Json;
use serde::Deserialize;

use crate::app_context::AppContext;
use arhiv::entities::Id;

#[derive(Deserialize)]
pub enum RPCAction {
    Delete { id: Id },
    Archive { id: Id, archive: bool },
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
    }

    Ok(Status::Ok)
}
