use std::fmt::Display;

use anyhow::{bail, Result};
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
        if InstanceId::is_valid_id(&self) {
            Ok(InstanceId(self))
        } else {
            bail!("Invalid InstanceId: {self}")
        }
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
    pub fn generate() -> Self {
        InstanceId(generate_random_id())
    }

    pub fn is_valid_id(value: &str) -> bool {
        !value.is_empty() && value.chars().all(|c| c.is_ascii_alphanumeric())
    }

    pub fn from_string(value: impl Into<String>) -> Result<Self> {
        value.into().try_into()
    }
}

impl Display for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}
