use anyhow::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RpcMessage {
    pub call_id: u32,
    pub action: String,
    pub params: Value,
}

impl std::str::FromStr for RpcMessage {
    type Err = anyhow::Error;

    fn from_str(data: &str) -> Result<RpcMessage> {
        serde_json::from_str(data).context("Failed to parse rpc message json")
    }
}

impl fmt::Display for RpcMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[RPC Message ({}): {} {}]",
            self.call_id, self.action, self.params,
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RpcMessageResponse {
    pub call_id: u32,
    pub result: Value,
    pub err: Option<String>,
}

impl RpcMessageResponse {
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize RpcMessageResponse to json")
    }
}
