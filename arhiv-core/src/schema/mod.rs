use crate::entities::{ATTACHMENT_TYPE, TOMBSTONE_TYPE};

pub use data_description::*;
pub use field::*;
pub use schema::*;

mod data_description;
mod field;
mod schema;
mod search;

impl DataSchema {
    #[must_use]
    pub fn new() -> DataSchema {
        DataSchema {
            modules: vec![
                // ----- INTERNAL
                DataDescription {
                    document_type: TOMBSTONE_TYPE,
                    is_internal: true,
                    collection_of: Collection::None,
                    fields: vec![],
                },
                DataDescription {
                    document_type: ATTACHMENT_TYPE,
                    is_internal: true,
                    collection_of: Collection::None,
                    fields: vec![
                        Field {
                            name: "filename",
                            field_type: FieldType::String {},
                            mandatory: true,
                        },
                        Field {
                            name: "sha256",
                            field_type: FieldType::String {},
                            mandatory: true,
                        },
                    ],
                },
                // ----
            ],
        }
    }
}

impl Default for DataSchema {
    fn default() -> Self {
        Self::new()
    }
}
