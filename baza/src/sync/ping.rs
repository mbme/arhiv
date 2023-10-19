use std::fmt;

use serde::{Deserialize, Serialize};

use rs_utils::Timestamp;

use crate::entities::{InstanceId, Revision};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Ping {
    pub timestamp: Timestamp,
    pub instance_id: InstanceId,
    pub data_version: u8,
    pub rev: Revision,
}

impl Ping {
    #[must_use]
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize Ping to json")
    }
}

impl fmt::Display for Ping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[PING from {}: {}]",
            self.instance_id,
            self.rev.serialize(),
        )
    }
}
