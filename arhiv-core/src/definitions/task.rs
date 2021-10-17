use crate::schema::*;

pub const PROJECT_TYPE: &str = "project";

pub const TASK_TYPE: &str = "task";

pub const TASK_STATUS: &[&str] = &[
    "Inbox",
    "InProgress",
    "Paused",
    "Todo",
    "Later",
    "Done",
    "Cancelled",
];

pub fn get_task_definitions() -> Vec<DataDescription> {
    vec![
        DataDescription {
            document_type: PROJECT_TYPE,
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
            document_type: TASK_TYPE,
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
                    field_type: FieldType::Enum(TASK_STATUS),
                    mandatory: true,
                },
                Field {
                    name: "project",
                    field_type: FieldType::Ref(PROJECT_TYPE),
                    mandatory: true,
                },
            ],
        },
    ]
}
