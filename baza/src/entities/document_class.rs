use std::fmt;

use serde::{Deserialize, Serialize};

pub const ERASED_DOCUMENT_TYPE: &str = "";

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct DocumentClass {
    pub document_type: String,
    pub subtype: String,
}

impl DocumentClass {
    pub fn erased() -> Self {
        DocumentClass {
            document_type: ERASED_DOCUMENT_TYPE.to_string(),
            subtype: "".to_string(),
        }
    }

    pub fn new(document_type: impl Into<String>, subtype: impl Into<String>) -> Self {
        DocumentClass {
            document_type: document_type.into(),
            subtype: subtype.into(),
        }
    }

    pub fn set_subtype(&mut self, subtype: impl Into<String>) {
        self.subtype = subtype.into();
    }

    #[must_use]
    pub fn is_erased(&self) -> bool {
        self.document_type == ERASED_DOCUMENT_TYPE
    }
}

impl fmt::Display for DocumentClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.document_type.as_str(), self.subtype.as_str()) {
            (ERASED_DOCUMENT_TYPE, _) => write!(f, "erased"),
            (document_type, "") => write!(f, "{document_type}"),
            (document_type, subtype) => write!(f, "{document_type}/{subtype}"),
        }
    }
}
