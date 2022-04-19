use std::ops::{Deref, DerefMut};

use anyhow::{ensure, Error, Result};
use serde_json::json;

use rs_utils::{get_file_name, get_file_size, get_media_type, FFProbe};

use crate::{
    entities::{BLOBId, Document},
    schema::*,
    ArhivConnection,
};

pub const ATTACHMENT_TYPE: &str = "attachment";
pub const AUDIO_SUBTYPE: &str = "audio";

const FIELD_FILENAME: &str = "filename";
const FIELD_MEDIA_TYPE: &str = "media_type";
const FIELD_BLOB: &str = "blob";
const FIELD_SIZE: &str = "size";

const DURATION: &str = "duration";
const BIT_RATE: &str = "bit_rate";

pub fn get_attachment_definitions() -> Vec<DataDescription> {
    vec![DataDescription {
        document_type: ATTACHMENT_TYPE,
        collection_of: Collection::None,
        fields: vec![
            Field {
                name: FIELD_FILENAME,
                field_type: FieldType::String {},
                mandatory: true,
                readonly: true,
                for_subtypes: None,
            },
            Field {
                name: FIELD_MEDIA_TYPE,
                field_type: FieldType::String {},
                mandatory: true,
                readonly: true,
                for_subtypes: None,
            },
            Field {
                name: FIELD_BLOB,
                field_type: FieldType::BLOBId,
                mandatory: true,
                readonly: true,
                for_subtypes: None,
            },
            Field {
                name: FIELD_SIZE, // in bytes
                field_type: FieldType::NaturalNumber {},
                mandatory: true,
                readonly: true,
                for_subtypes: None,
            },
            Field {
                name: DURATION, // in milliseconds
                field_type: FieldType::NaturalNumber {},
                mandatory: true,
                readonly: true,
                for_subtypes: Some(&[AUDIO_SUBTYPE]),
            },
            Field {
                name: BIT_RATE,
                field_type: FieldType::NaturalNumber {},
                mandatory: true,
                readonly: true,
                for_subtypes: Some(&[AUDIO_SUBTYPE]),
            },
        ],
        subtypes: Some(&["", AUDIO_SUBTYPE]),
    }]
}

#[derive(Debug)]
pub struct Attachment(Document);

impl Attachment {
    #[must_use]
    pub fn is_attachment(document: &Document) -> bool {
        document.document_type == ATTACHMENT_TYPE
    }

    fn new(filename: &str, media_type: &str, size: u64) -> Self {
        let document = Document::new_with_data(
            ATTACHMENT_TYPE,
            "",
            json!({
                FIELD_FILENAME: filename,
                FIELD_MEDIA_TYPE: media_type,
                FIELD_SIZE: size,
            })
            .try_into()
            .expect("failed to serialize data"),
        );

        Attachment(document)
    }

    pub fn create(file_path: &str) -> Result<Self> {
        let filename = get_file_name(file_path).to_string();
        let media_type = get_media_type(file_path)?;
        let size = get_file_size(file_path)?;

        let mut attachment = Attachment::new(&filename, &media_type, size);

        if attachment.is_audio() {
            let ffprobe = FFProbe::check()?;

            let stats = ffprobe.get_stats(file_path)?;

            attachment.subtype = AUDIO_SUBTYPE.to_string();
            attachment.data.set(DURATION, stats.duration_ms);
            attachment.data.set(BIT_RATE, stats.bit_rate);
        }

        Ok(attachment)
    }

    pub fn create_and_stage(
        file_path: &str,
        move_file: bool,
        tx: &mut ArhivConnection,
    ) -> Result<Self> {
        let mut attachment = Attachment::create(file_path)?;
        let blob_id = tx.add_blob(file_path, move_file)?;
        attachment.set_blob_id(blob_id);

        Ok(attachment)
    }

    #[must_use]
    pub fn get_blob_id(&self) -> BLOBId {
        let blob_id = self.data.get_mandatory_str(FIELD_BLOB);

        BLOBId::from_string(blob_id)
    }

    pub fn set_blob_id(&mut self, blob_id: BLOBId) {
        self.data.set(FIELD_BLOB, blob_id);
    }

    #[must_use]
    pub fn get_filename(&self) -> &str {
        self.data.get_mandatory_str(FIELD_FILENAME)
    }

    pub fn set_filename(&mut self, filename: &str) {
        self.data.set(FIELD_FILENAME, filename);
    }

    #[must_use]
    pub fn get_media_type(&self) -> &str {
        self.data.get_mandatory_str(FIELD_MEDIA_TYPE)
    }

    #[must_use]
    pub fn is_image(&self) -> bool {
        self.get_media_type().starts_with("image/")
    }

    #[must_use]
    pub fn is_audio(&self) -> bool {
        self.get_media_type().starts_with("audio/")
    }
}

impl Deref for Attachment {
    type Target = Document;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Attachment {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Attachment> for Document {
    fn from(val: Attachment) -> Self {
        val.0
    }
}

impl TryInto<Attachment> for Document {
    type Error = Error;

    fn try_into(self) -> Result<Attachment, Self::Error> {
        ensure!(
            Attachment::is_attachment(&self),
            "document {} must be an attachment",
            &self.id,
        );

        Ok(Attachment(self))
    }
}
