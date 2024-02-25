use baza::schema::*;

use super::fields::*;

pub const FILM_TYPE: &str = "film";
pub const FILM_COLLECTION_TYPE: &str = "film collection";

#[allow(clippy::too_many_lines)]
pub fn get_film_definitions() -> Vec<DataDescription> {
    vec![
        DataDescription {
            document_type: FILM_TYPE,
            title_format: "{title} ({release_date})",
            fields: vec![
                Field {
                    name: "title",
                    field_type: FieldType::String {},
                    mandatory: true,
                    readonly: false,
                },
                Field {
                    name: "cover",
                    field_type: FieldType::Ref(&[ATTACHMENT_TYPE]),
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
                    name: "language",
                    field_type: LANGUAGE_FIELD,
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "original_language",
                    field_type: LANGUAGE_FIELD,
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "countries_of_origin",
                    field_type: FieldType::Countries {},
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "creators",
                    field_type: FieldType::People {},
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "cast",
                    field_type: FieldType::People {},
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "duration",
                    field_type: FieldType::Duration {},
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "seasons",
                    field_type: FieldType::NaturalNumber {},
                    mandatory: false,
                    readonly: false,
                },
                Field {
                    name: "episodes",
                    field_type: FieldType::NaturalNumber {},
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
            document_type: FILM_COLLECTION_TYPE,
            title_format: "{name}",
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
                    name: "films",
                    field_type: FieldType::RefList(&[FILM_TYPE]),
                    mandatory: false,
                    readonly: false,
                },
            ],
        },
    ]
}
