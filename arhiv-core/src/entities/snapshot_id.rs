use std::fmt;
use std::ops::Deref;

use serde::{Deserialize, Serialize};

use rs_utils::gen_uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct SnapshotId(String);

impl SnapshotId {
    #[must_use]
    pub fn new() -> SnapshotId {
        SnapshotId(gen_uuid())
    }
}

impl Default for SnapshotId {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for SnapshotId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for SnapshotId {
    fn from(value: &str) -> Self {
        SnapshotId(value.to_string())
    }
}

impl From<String> for SnapshotId {
    fn from(value: String) -> Self {
        SnapshotId(value)
    }
}

impl fmt::Display for SnapshotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
