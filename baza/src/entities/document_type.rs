use std::{fmt, ops::Deref};

use serde::{Deserialize, Serialize};

pub const ERASED_DOCUMENT_TYPE: &str = "";

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct DocumentType(String);

impl DocumentType {
    pub fn erased() -> Self {
        DocumentType(ERASED_DOCUMENT_TYPE.to_string())
    }

    pub fn new(document_type: impl Into<String>) -> Self {
        DocumentType(document_type.into())
    }

    #[must_use]
    pub fn is_erased(&self) -> bool {
        self.0 == ERASED_DOCUMENT_TYPE
    }

    #[must_use]
    pub fn is(&self, expected_type: &str) -> bool {
        self.0 == expected_type
    }
}

impl Deref for DocumentType {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for DocumentType {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<DocumentType> for String {
    fn from(value: DocumentType) -> Self {
        value.0
    }
}

impl PartialEq<&str> for DocumentType {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_str() == *other
    }
}

impl fmt::Display for DocumentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.as_str() {
            ERASED_DOCUMENT_TYPE => write!(f, "erased"),
            document_type => write!(f, "{document_type}"),
        }
    }
}
