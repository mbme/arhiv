use super::fields::*;
use super::ATTACHMENT_TYPE;
use crate::schema::*;

pub const TRACK_TYPE: &str = "track";
pub const TRACK_COLLECTION_TYPE: &str = "track collection";

#[allow(clippy::too_many_lines)]
pub fn get_track_definitions() -> Vec<DataDescription> {
    vec![
        DataDescription {
            document_type: TRACK_TYPE,
            collection_of: Collection::None,
            fields: vec![
                Field {
                    name: "title",
                    field_type: FieldType::String {},
                    mandatory: true,
                    readonly: false,
                    for_subtypes: &[],
                },
                Field {
                    name: "artist",
                    field_type: FieldType::String {},
                    mandatory: true,
                    readonly: false,
                    for_subtypes: &[],
                },
                Field {
                    name: "track",
                    field_type: FieldType::Ref(ATTACHMENT_TYPE),
                    mandatory: true,
                    readonly: false,
                    for_subtypes: &[],
                },
                Field {
                    name: "cover",
                    field_type: FieldType::Ref(ATTACHMENT_TYPE),
                    mandatory: false,
                    readonly: false,
                    for_subtypes: &[],
                },
                Field {
                    name: "release_date",
                    field_type: FieldType::Date {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: &[],
                },
                Field {
                    name: "language",
                    field_type: LANGUAGE_FIELD,
                    mandatory: false,
                    readonly: false,
                    for_subtypes: &[],
                },
                Field {
                    name: "collections",
                    field_type: FieldType::RefList(TRACK_COLLECTION_TYPE),
                    mandatory: false,
                    readonly: false,
                    for_subtypes: &[],
                },
                Field {
                    name: "comment",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: &[],
                },
            ],
            subtypes: &[],
        },
        DataDescription {
            document_type: TRACK_COLLECTION_TYPE,
            collection_of: Collection::Type {
                document_type: TRACK_TYPE,
                field: "collections",
            },
            fields: vec![
                Field {
                    name: "name",
                    field_type: FieldType::String {},
                    mandatory: true,
                    readonly: false,
                    for_subtypes: &[],
                },
                Field {
                    name: "description",
                    field_type: FieldType::MarkupString {},
                    mandatory: false,
                    readonly: false,
                    for_subtypes: &[],
                },
            ],
            subtypes: &[],
        },
    ]
}
