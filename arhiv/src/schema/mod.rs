mod data_description;

pub use data_description::*;
use lazy_static::*;

lazy_static! {
    pub static ref SCHEMA: DataSchema = DataSchema {
        version: 1,
        modules: vec![
            DataDescription {
                document_type: "attachment",
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
                fields: vec![
                    Field {
                        name: "name",
                        field_type: FieldType::String {},
                    },
                    Field {
                        name: "data",
                        field_type: FieldType::MarkupString {},
                    },
                ],
            },
            DataDescription {
                document_type: "project",
                collection_of: Some(Collection { item_type: "task" }),
                fields: vec![
                    Field {
                        name: "title",
                        field_type: FieldType::String {},
                    },
                    Field {
                        name: "description",
                        field_type: FieldType::MarkupString {},
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
                    },
                    Field {
                        name: "title",
                        field_type: FieldType::String {},
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
        ],
    };
}
