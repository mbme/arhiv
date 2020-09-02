use anyhow::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
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
