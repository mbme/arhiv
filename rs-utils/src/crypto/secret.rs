use std::borrow::Borrow;

use secstr::{SecUtf8, SecVec};
use serde::Deserialize;

#[derive(Clone)]
pub struct SecretBytes(SecVec<u8>);

impl SecretBytes {
    pub fn new(value: Vec<u8>) -> Self {
        Self(SecVec::new(value))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.borrow()
    }

    pub fn len(&self) -> usize {
        self.as_bytes().len()
    }

    pub fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }
}

impl AsRef<[u8]> for SecretBytes {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<Vec<u8>> for SecretBytes {
    fn from(value: Vec<u8>) -> Self {
        Self::new(value)
    }
}

impl From<&[u8]> for SecretBytes {
    fn from(value: &[u8]) -> Self {
        Self::new(value.into())
    }
}

impl From<&str> for SecretBytes {
    fn from(value: &str) -> Self {
        Self::new(value.as_bytes().to_vec())
    }
}

impl From<SecretString> for SecretBytes {
    fn from(value: SecretString) -> Self {
        value.as_ref().into()
    }
}

#[derive(Deserialize)]
pub struct SecretString(SecUtf8);

impl SecretString {
    pub fn new(value: impl Into<String>) -> Self {
        Self(SecUtf8::from(value))
    }

    pub fn as_str(&self) -> &str {
        self.0.unsecure()
    }

    pub fn into_unsecure_string(self) -> String {
        self.0.into_unsecure()
    }

    pub fn len(&self) -> usize {
        self.as_str().len()
    }

    pub fn is_empty(&self) -> bool {
        self.as_str().is_empty()
    }
}

impl From<String> for SecretString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl AsRef<[u8]> for SecretString {
    fn as_ref(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}
