use baza::schema::*;

use super::fields::*;

pub const GAME_TYPE: &str = "game";
pub const GAME_COLLECTION_TYPE: &str = "game collection";

#[allow(clippy::too_many_lines)]
pub fn get_game_definitions() -> Vec<DataDescription> {
    vec![
        DataDescription {
            document_type: GAME_TYPE,
            title_format: "${name} (${release_date})",
            fields: vec![
                Field {
                    name: "name",
                    field_type: FieldType::String {},
                    mandatory: true,
                    readonly: false,
                },
                Field {
                    name: "cover",
                    field_type: FieldType::Ref(&[ASSET_TYPE]),
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "release_date",
                    field_type: FieldType::Date {},
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "developers",
                    field_type: FieldType::String {},
                    mandatory: true,
                    readonly: false,
                },
                Field {
                    name: "language",
                    field_type: LANGUAGE_FIELD,
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "description",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "status",
                    field_type: STATUS_FIELD,
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "rating",
                    field_type: RATING_FIELD,
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "comment",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                },
            ],
        },
        DataDescription {
            document_type: GAME_COLLECTION_TYPE,
            title_format: "${name}",
            fields: vec![
                Field {
                    name: "name",
                    field_type: FieldType::String {},
                    mandatory: true,
                    readonly: false,
                },
                Field {
                    name: "description",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "games",
                    field_type: FieldType::RefList(&[GAME_TYPE]),
                    mandatory: false,
                    readonly: false,
                },
            ],
        },
    ]
}
