use serde::{Deserialize, Serialize};
use std::convert::From;
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct Revision(pub u32);

impl Revision {
    pub const STAGING: Revision = Revision(0);

    pub fn from(value: u32) -> Self {
        Revision(value)
    }

    pub fn is_staged(&self) -> bool {
        self.0 == 0
    }

    pub fn is_committed(&self) -> bool {
        self.0 > 0
    }

    pub fn inc(&self) -> Self {
        Revision(self.0 + 1)
    }
}

impl fmt::Display for Revision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for Revision {
    fn from(value: u32) -> Self {
        Revision(value)
    }
}

impl From<Revision> for u32 {
    fn from(value: Revision) -> Self {
        value.0
    }
}
