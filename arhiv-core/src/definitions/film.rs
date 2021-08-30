use super::fields::*;
use crate::{entities::ATTACHMENT_TYPE, schema::*};

#[allow(clippy::too_many_lines)]
pub fn get_film_definitions() -> Vec<DataDescription> {
    vec![
        DataDescription {
            document_type: "film",
            is_internal: false,
            collection_of: Collection::None,
            fields: vec![
                Field {
                    name: "title",
                    field_type: FieldType::String {},
                    mandatory: true,
                },
                Field {
                    name: "cover",
                    field_type: FieldType::Ref(ATTACHMENT_TYPE),
                    mandatory: false,
                },
                Field {
                    name: "release_date",
                    field_type: FieldType::Date {},
                    mandatory: false,
                },
                Field {
                    name: "language",
                    field_type: language_field(),
                    mandatory: false,
                },
                Field {
                    name: "original_language",
                    field_type: language_field(),
                    mandatory: false,
                },
                Field {
                    name: "countries_of_origin",
                    field_type: FieldType::Countries {},
                    mandatory: false,
                },
                Field {
                    name: "creators",
                    field_type: FieldType::People {},
                    mandatory: false,
                },
                Field {
                    name: "cast",
                    field_type: FieldType::People {},
                    mandatory: false,
                },
                Field {
                    name: "duration",
                    field_type: FieldType::Duration {},
                    mandatory: false,
                },
                Field {
                    name: "is_series",
                    field_type: FieldType::Flag {},
                    mandatory: true,
                },
                // ----------- if Series
                Field {
                    name: "number_of_seasons",
                    field_type: FieldType::NaturalNumber {},
                    mandatory: false,
                },
                Field {
                    name: "number_of_episodes",
                    field_type: FieldType::NaturalNumber {},
                    mandatory: false,
                },
                Field {
                    name: "episode_duration",
                    field_type: FieldType::Duration {},
                    mandatory: false,
                },
                // -----------
                Field {
                    name: "description",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                },
                Field {
                    name: "collections",
                    field_type: FieldType::RefList("film collection"),
                    mandatory: false,
                },
                Field {
                    name: "completed",
                    field_type: FieldType::Flag {},
                    mandatory: false,
                },
                Field {
                    name: "rating",
                    field_type: rating_field(),
                    mandatory: false,
                },
                Field {
                    name: "comment",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                },
            ],
        },
        DataDescription {
            document_type: "film collection",
            is_internal: false,
            collection_of: Collection::Type {
                document_type: "film",
                field: "collections",
            },
            fields: vec![
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
    ]
}
