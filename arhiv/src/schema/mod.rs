use lazy_static::*;

use crate::entities::ATTACHMENT_TYPE;
use crate::entities::TOMBSTONE_TYPE;
pub use data_description::*;

mod data_description;

lazy_static! {
    pub static ref SCHEMA: DataSchema = DataSchema {
        modules: vec![
            DataDescription {
                document_type: TOMBSTONE_TYPE,
                collection_of: None,
                fields: vec![],
            },
            DataDescription {
                document_type: ATTACHMENT_TYPE,
                collection_of: None,
                fields: vec![
                    Field {
                        name: "filename",
                        field_type: FieldType::String {},
                    },
                    Field {
                        name: "hash",
                        field_type: FieldType::String {},
                    },
                ],
            },
            DataDescription {
                document_type: "note",
                collection_of: None,
                fields: vec![Field {
                    name: "data",
                    field_type: FieldType::MarkupString {},
                }],
            },
            DataDescription {
                document_type: "project",
                collection_of: Some(Collection { item_type: "task" }),
                fields: vec![Field {
                    name: "description",
                    field_type: FieldType::MarkupString {},
                },],
            },
            DataDescription {
                document_type: "task",
                collection_of: None,
                fields: vec![
                    Field {
                        name: "project",
                        field_type: FieldType::Ref("project"),
                    },
                    Field {
                        name: "description",
                        field_type: FieldType::MarkupString {},
                    },
                    Field {
                        name: "complexity",
                        field_type: FieldType::Enum(vec![
                            "Unknown", "Small", "Medium", "Large", "Epic",
                        ]),
                    },
                    Field {
                        name: "status",
                        field_type: FieldType::Enum(vec![
                            "Inbox",
                            "Todo",
                            "Later",
                            "InProgress",
                            "Paused",
                            "Done",
                            "Cancelled",
                        ]),
                    },
                ],
            },
            DataDescription {
                document_type: "book",
                collection_of: None,
                fields: vec![
                    Field {
                        name: "title",
                        field_type: FieldType::String {},
                    },
                    Field {
                        name: "authors",
                        field_type: FieldType::String {},
                    },
                    Field {
                        name: "cover",
                        field_type: FieldType::Ref(ATTACHMENT_TYPE),
                    },
                    Field {
                        name: "language",
                        field_type: FieldType::Enum(vec![
                            "undefined",
                            "Ukrainian",
                            "English",
                            "Russian",
                        ]),
                    },
                    Field {
                        name: "translators",
                        field_type: FieldType::String {},
                    },
                    Field {
                        name: "publisher",
                        field_type: FieldType::String {},
                    },
                    Field {
                        name: "publication_date",
                        field_type: FieldType::String {},
                    },
                    Field {
                        name: "description",
                        field_type: FieldType::MarkupString {},
                    },
                    Field {
                        name: "ISBN",
                        field_type: FieldType::String {},
                    },
                    Field {
                        name: "rating",
                        field_type: FieldType::Enum(vec![
                            "Unknown", //
                            "Very Bad",
                            "Bad",
                            "Average",
                            "Fine",
                            "Good",
                            "Great",
                        ]),
                    },
                    Field {
                        name: "comment",
                        field_type: FieldType::MarkupString {},
                    },
                ],
            },
        ],
    };
}
