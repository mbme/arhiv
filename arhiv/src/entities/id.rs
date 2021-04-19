use serde::{Deserialize, Serialize};
use std::fmt;

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

    pub fn from_string(id: String) -> Self {
        Id(id)
    }

    pub fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
