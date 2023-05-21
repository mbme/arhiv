use std::fmt;

use serde::{Deserialize, Serialize};

use rs_utils::generate_random_id;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct InstanceId(String);

impl fmt::Display for InstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<InstanceId> for String {
    fn from(value: InstanceId) -> Self {
        value.0
    }
}

impl InstanceId {
    pub fn new() -> Self {
        InstanceId(generate_random_id())
    }

    #[must_use]
    pub fn from_string(instance_id: impl Into<String>) -> Self {
        InstanceId(instance_id.into())
    }
}

impl Default for InstanceId {
    fn default() -> Self {
        InstanceId::new()
    }
}
