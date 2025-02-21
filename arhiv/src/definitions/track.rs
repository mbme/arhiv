use serde::{Deserialize, Serialize};

use baza::{
    entities::{Document, Id},
    schema::*,
};

use super::fields::*;

pub const TRACK_TYPE: &str = "track";
pub const TRACK_COLLECTION_TYPE: &str = "track collection";

#[allow(clippy::too_many_lines)]
pub fn get_track_definitions() -> Vec<DataDescription> {
    vec![
        DataDescription {
            document_type: TRACK_TYPE,
            title_format: "{artist} - {title}",
            fields: vec![
                Field {
                    name: "title",
                    field_type: FieldType::String {},
                    mandatory: true,
                    readonly: false,
                },
                Field {
                    name: "artist",
                    field_type: FieldType::String {},
                    mandatory: true,
                    readonly: false,
                },
                Field {
                    name: "track",
                    field_type: FieldType::Ref(&[ASSET_TYPE]),
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
                    name: "language",
                    field_type: LANGUAGE_FIELD,
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
            document_type: TRACK_COLLECTION_TYPE,
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
                    name: "tracks",
                    field_type: FieldType::RefList(&[TRACK_TYPE]),
                    mandatory: false,
                    readonly: false,
                },
            ],
        },
    ]
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TrackData {
    pub title: String,
    pub artist: String,
    pub track: Id,
    pub cover: Option<Id>,
    pub release_date: Option<String>,
    pub language: Option<String>,
    pub comment: Option<String>,
}

pub type TrackDocument = Document<TrackData>;
