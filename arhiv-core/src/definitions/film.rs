use super::fields::*;
use crate::{entities::ATTACHMENT_TYPE, schema::*};

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
                    optional: false,
                },
                Field {
                    name: "cover",
                    field_type: FieldType::Ref(ATTACHMENT_TYPE),
                    optional: true,
                },
                Field {
                    name: "release_date",
                    field_type: FieldType::Date {},
                    optional: true,
                },
                Field {
                    name: "language",
                    field_type: language_field(),
                    optional: true,
                },
                Field {
                    name: "original_language",
                    field_type: language_field(),
                    optional: true,
                },
                Field {
                    name: "countries_of_origin",
                    field_type: FieldType::Countries {},
                    optional: true,
                },
                Field {
                    name: "creators",
                    field_type: FieldType::People {},
                    optional: true,
                },
                Field {
                    name: "cast",
                    field_type: FieldType::People {},
                    optional: true,
                },
                Field {
                    name: "duration",
                    field_type: FieldType::Duration {},
                    optional: true,
                },
                Field {
                    name: "is_series",
                    field_type: FieldType::Flag {},
                    optional: false,
                },
                // ----------- if Series
                Field {
                    name: "number_of_seasons",
                    field_type: FieldType::NaturalNumber {},
                    optional: true,
                },
                Field {
                    name: "number_of_episodes",
                    field_type: FieldType::NaturalNumber {},
                    optional: true,
                },
                Field {
                    name: "episode_duration",
                    field_type: FieldType::Duration {},
                    optional: true,
                },
                // -----------
                Field {
                    name: "description",
                    field_type: FieldType::MarkupString {},
                    optional: true,
                },
                Field {
                    name: "collections",
                    field_type: FieldType::RefList("film collection"),
                    optional: true,
                },
                Field {
                    name: "completed",
                    field_type: FieldType::Flag {},
                    optional: true,
                },
                Field {
                    name: "rating",
                    field_type: rating_field(),
                    optional: true,
                },
                Field {
                    name: "comment",
                    field_type: FieldType::MarkupString {},
                    optional: true,
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
                    optional: false,
                },
                Field {
                    name: "description",
                    field_type: FieldType::MarkupString {},
                    optional: true,
                },
            ],
        },
    ]
}
