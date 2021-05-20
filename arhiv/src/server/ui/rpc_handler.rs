use std::fmt;
use std::sync::Arc;

use anyhow::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use warp::{Rejection, Reply};

use crate::commander::ArhivCommander;
use rs_utils::log::debug;

pub async fn rpc_action_handler(
    commander: Arc<ArhivCommander>,
    msg: RpcMessage,
) -> Result<impl Reply, Rejection> {
    debug!("RPC MESSAGE: {}", msg);

    let result = commander.run(msg.action, msg.params).await;

    let response = match result {
        Ok(result) => RpcMessageResponse { result, err: None },
        Err(err) => RpcMessageResponse {
            result: serde_json::Value::Null,
            err: Some(err.to_string()),
        },
    };

    Ok(warp::reply::json(&response).into_response())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RpcMessage {
    pub action: String,
    pub params: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RpcMessageResponse {
    pub result: Value,
    pub err: Option<String>,
}

impl fmt::Display for RpcMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[RPC Message: {} {}]", self.action, self.params,)
    }
}
