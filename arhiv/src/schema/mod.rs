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
                        optional: false,
                    },
                    Field {
                        name: "hash",
                        field_type: FieldType::Hash {},
                        optional: false,
                    },
                ],
            },
            DataDescription {
                document_type: "note",
                collection_of: None,
                fields: vec![ //
                    Field {
                        name: "data",
                        field_type: FieldType::MarkupString {},
                        optional: false,
                    },
                ],
            },
            DataDescription {
                document_type: "project",
                collection_of: Some(Collection { item_type: "task" }),
                fields: vec![ //
                    Field {
                        name: "description",
                        field_type: FieldType::MarkupString {},
                        optional: true,
                    },
                ],
            },
            DataDescription {
                document_type: "task",
                collection_of: None,
                fields: vec![
                    Field {
                        name: "project",
                        field_type: FieldType::Ref("project"),
                        optional: false,
                    },
                    Field {
                        name: "description",
                        field_type: FieldType::MarkupString {},
                        optional: false,
                    },
                    Field {
                        name: "complexity",
                        field_type: FieldType::Enum(vec![
                            "Unknown", //
                            "Small",
                            "Medium",
                            "Large",
                            "Epic",
                        ]),
                        optional: false,
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
                        optional: false,
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
                        optional: false,
                    },
                    Field {
                        name: "authors",
                        field_type: FieldType::String {},
                        optional: false,
                    },
                    Field {
                        name: "cover",
                        field_type: FieldType::Ref(ATTACHMENT_TYPE),
                        optional: true,
                    },
                    Field {
                        name: "language",
                        field_type: FieldType::Enum(vec![
                            "Ukrainian",
                            "English",
                            "Russian",
                        ]),
                        optional: true,
                    },
                    Field {
                        name: "translators",
                        field_type: FieldType::String {},
                        optional: true,
                    },
                    Field {
                        name: "publisher",
                        field_type: FieldType::String {},
                        optional: true,
                    },
                    Field {
                        name: "publication_date",
                        field_type: FieldType::Date {},
                        optional: true,
                    },
                    Field {
                        name: "description",
                        field_type: FieldType::MarkupString {},
                        optional: true,
                    },
                    Field {
                        name: "ISBN",
                        field_type: FieldType::ISBN {},
                        optional: true,
                    },
                    Field {
                        name: "rating",
                        field_type: FieldType::Enum(vec![
                            "Very Bad", //
                            "Bad",
                            "Average",
                            "Fine",
                            "Good",
                            "Great",
                        ]),
                        optional: true,
                    },
                    Field {
                        name: "comment",
                        field_type: FieldType::MarkupString {},
                        optional: true,
                    },
                ],
            },
        ],
    };
}
