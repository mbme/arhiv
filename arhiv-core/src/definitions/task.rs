use crate::schema::*;

pub fn get_task_definitions() -> Vec<DataDescription> {
    vec![
        DataDescription {
            document_type: "project",
            is_internal: false,
            collection_of: Collection::Type {
                document_type: "task",
                field: "project",
            },
            fields: vec![
                //
                Field {
                    name: "name",
                    field_type: FieldType::String {},
                    mandatory: true,
                },
                Field {
                    name: "description",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                },
            ],
        },
        DataDescription {
            document_type: "task",
            is_internal: false,
            collection_of: Collection::None,
            fields: vec![
                Field {
                    name: "title",
                    field_type: FieldType::String {},
                    mandatory: true,
                },
                Field {
                    name: "description",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                },
                Field {
                    name: "status",
                    field_type: FieldType::Enum(vec![
                        "Inbox",
                        "InProgress",
                        "Paused",
                        "Todo",
                        "Later",
                        "Done",
                        "Cancelled",
                    ]),
                    mandatory: true,
                },
                Field {
                    name: "project",
                    field_type: FieldType::Ref("project"),
                    mandatory: true,
                },
            ],
        },
    ]
}
