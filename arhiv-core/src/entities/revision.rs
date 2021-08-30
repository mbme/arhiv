use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct Revision(pub u32);

impl Revision {
    pub const STAGING: Revision = Revision(0);

    #[must_use]
    pub fn from_value(value: u32) -> Self {
        Revision(value)
    }

    #[must_use]
    pub fn is_staged(&self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub fn is_committed(&self) -> bool {
        self.0 > 0
    }

    #[must_use]
    pub fn inc(&self) -> Self {
        Revision(self.0 + 1)
    }
}

impl fmt::Display for Revision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
