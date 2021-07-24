use std::fmt;
use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct Id(String);

impl Id {
    pub fn new() -> Self {
        // TODO make const fn
        let chars: Vec<char> = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
            .chars()
            .collect();

        // see https://zelark.github.io/nano-id-cc/
        Id(nanoid::nanoid!(14, &chars))
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

impl From<&Id> for Id {
    fn from(value: &Id) -> Self {
        value.clone()
    }
}

impl From<Id> for String {
    fn from(value: Id) -> Self {
        value.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
