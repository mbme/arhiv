use std::fmt::Display;

use serde::{Deserialize, Serialize};

use rs_utils::generate_random_id;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct InstanceId(String);

impl From<InstanceId> for String {
    fn from(value: InstanceId) -> Self {
        value.0
    }
}

impl TryInto<InstanceId> for String {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<InstanceId, Self::Error> {
        // FIXME check if valid id [a-z0-9]
        Ok(InstanceId(self))
    }
}

impl TryInto<InstanceId> for &str {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<InstanceId, Self::Error> {
        self.to_string().try_into()
    }
}

impl AsRef<str> for InstanceId {
    fn as_ref(&self) -> &str {
        &self.0
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

impl Display for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}
