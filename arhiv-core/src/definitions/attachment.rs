use anyhow::Result;
use serde::{Deserialize, Serialize};

use rs_utils::{get_file_name, get_file_size, get_image_size, get_media_type, log, FFProbe};

use baza::{
    entities::{BLOBId, Document},
    schema::*,
    BazaConnection,
};

pub const ATTACHMENT_TYPE: &str = "attachment";
pub const AUDIO_SUBTYPE: &str = "audio";
pub const IMAGE_SUBTYPE: &str = "image";

pub fn get_attachment_definitions() -> Vec<DataDescription> {
    vec![DataDescription {
        document_type: ATTACHMENT_TYPE,
        fields: vec![
            Field {
                name: "filename",
                field_type: FieldType::String {},
                mandatory: true,
                readonly: false,
                for_subtypes: None,
            },
            Field {
                name: "media_type",
                field_type: FieldType::String {},
                mandatory: true,
                readonly: false,
                for_subtypes: None,
            },
            Field {
                name: "blob",
                field_type: FieldType::BLOBId {},
                mandatory: true,
                readonly: true,
                for_subtypes: None,
            },
            Field {
                name: "size", // in bytes
                field_type: FieldType::NaturalNumber {},
                mandatory: true,
                readonly: true,
                for_subtypes: None,
            },
            Field {
                name: "duration", // in milliseconds
                field_type: FieldType::NaturalNumber {},
                mandatory: false,
                readonly: false,
                for_subtypes: Some(&[AUDIO_SUBTYPE]),
            },
            Field {
                name: "bit_rate",
                field_type: FieldType::NaturalNumber {},
                mandatory: false,
                readonly: false,
                for_subtypes: Some(&[AUDIO_SUBTYPE]),
            },
            Field {
                name: "width",
                field_type: FieldType::NaturalNumber {},
                mandatory: false,
                readonly: false,
                for_subtypes: Some(&[IMAGE_SUBTYPE]),
            },
            Field {
                name: "height",
                field_type: FieldType::NaturalNumber {},
                mandatory: false,
                readonly: false,
                for_subtypes: Some(&[IMAGE_SUBTYPE]),
            },
        ],
        subtypes: Some(&["", AUDIO_SUBTYPE, IMAGE_SUBTYPE]),
    }]
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AttachmentData {
    pub filename: String,
    pub media_type: String,
    pub blob: BLOBId,
    pub size: u64,

    // if audio
    pub duration: Option<u64>,
    pub bit_rate: Option<u64>,

    // if image
    pub width: Option<u64>,
    pub height: Option<u64>,
}

impl AttachmentData {
    #[must_use]
    pub fn is_image(&self) -> bool {
        self.media_type.starts_with("image/")
    }

    #[must_use]
    pub fn is_audio(&self) -> bool {
        self.media_type.starts_with("audio/")
    }
}

pub type Attachment = Document<AttachmentData>;

pub fn create_attachment(
    file_path: &str,
    move_file: bool,
    tx: &mut BazaConnection,
) -> Result<Attachment> {
    let filename = get_file_name(file_path).to_string();
    let media_type = get_media_type(file_path)?;
    let size = get_file_size(file_path)?;

    let blob = tx.add_blob(file_path, move_file)?;

    let mut attachment = Document::new_with_data(
        ATTACHMENT_TYPE,
        "",
        AttachmentData {
            filename,
            media_type,
            size,
            blob,
            duration: None,
            bit_rate: None,
            width: None,
            height: None,
        },
    );

    if attachment.data.is_audio() {
        attachment.subtype = AUDIO_SUBTYPE.to_string();

        let stats = FFProbe::check().and_then(|ffprobe| ffprobe.get_stats(file_path));

        match stats {
            Ok(stats) => {
                attachment.data.duration = Some(stats.duration_ms as u64);
                attachment.data.bit_rate = Some(stats.bit_rate as u64);
            }
            Err(err) => {
                log::warn!("Failed to get audio stats from file {}: {}", file_path, err);
            }
        }
    }

    if attachment.data.is_image() {
        attachment.subtype = IMAGE_SUBTYPE.to_string();

        match get_image_size(file_path) {
            Ok((width, height)) => {
                attachment.data.width = Some(width as u64);
                attachment.data.height = Some(height as u64);
            }
            Err(err) => {
                log::warn!("Failed to get image size from file {}: {}", file_path, err);
            }
        }
    }

    let mut document = attachment.into_document()?;
    tx.stage_document(&mut document)?;

    document.convert()
}
