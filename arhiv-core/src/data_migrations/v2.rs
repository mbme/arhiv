use std::borrow::Cow;

use anyhow::Result;
use rs_utils::{get_image_size, log};

use baza::{
    entities::{BLOBId, Document, BLOB},
    schema::DataMigration,
    BazaConnection,
};

pub struct DataSchema2;

impl DataMigration for DataSchema2 {
    fn get_version(&self) -> u8 {
        2
    }

    fn update(&self, document: &mut Cow<Document>, conn: &BazaConnection) -> Result<()> {
        // in attachment
        // if image, add subtype and dimensions
        if document.document_type == "attachment"
            && document
                .data
                .get_mandatory_str("media_type")
                .starts_with("image/")
        {
            let mut document = document.to_mut();
            document.subtype = "image".to_string();

            let blob_id = BLOBId::from_string(document.data.get_mandatory_str("blob"));
            let blob = BLOB::new(blob_id, &conn.get_path_manager().data_dir);

            match get_image_size(&blob.file_path) {
                Ok((width, height)) => {
                    document.data.set("width", width);
                    document.data.set("height", height);
                }
                Err(err) => {
                    log::warn!(
                        "Failed to get image size from file {}: {}",
                        blob.file_path,
                        err
                    );
                }
            }
        }

        Ok(())
    }
}
