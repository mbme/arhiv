use serde::{Deserialize, Serialize};
use std::convert::From;
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct Id(pub String);

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

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Id {
    fn from(value: String) -> Self {
        Id(value)
    }
}

impl From<&str> for Id {
    fn from(value: &str) -> Self {
        Id(value.to_string())
    }
}

impl From<Id> for String {
    fn from(value: Id) -> Self {
        value.0
    }
}

impl<'a> From<&'a Id> for &'a str {
    fn from(value: &'a Id) -> &'a str {
        &value.0
    }
}
