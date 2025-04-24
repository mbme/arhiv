use std::fmt;
use std::ops::Deref;

use serde::{Deserialize, Serialize};

use rs_utils::generate_random_id;

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct Id(String);

impl Id {
    #[must_use]
    pub fn new() -> Self {
        Id(generate_random_id())
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Id {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for Id {
    fn from(value: &str) -> Self {
        Id(value.to_string())
    }
}

impl From<String> for Id {
    fn from(value: String) -> Self {
        Id(value)
    }
}

impl From<&String> for Id {
    fn from(value: &String) -> Self {
        Id(value.clone())
    }
}

impl From<Id> for String {
    fn from(value: Id) -> Self {
        value.0
    }
}

impl From<&Id> for String {
    fn from(value: &Id) -> Self {
        value.0.clone()
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<id: {}>", self.0)
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
