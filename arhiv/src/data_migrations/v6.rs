use std::borrow::Cow;

use anyhow::Result;

use baza::{entities::Document, schema::DataMigration, BazaConnection};

pub struct DataSchema6;

/// Remove attachment subtype fields
impl DataMigration for DataSchema6 {
    fn get_version(&self) -> u8 {
        6
    }

    fn update(&self, document: &mut Cow<Document>, _conn: &BazaConnection) -> Result<()> {
        if document.document_type.is("attachment") {
            let document = document.to_mut();
            document.data.remove("duration");
            document.data.remove("bit_rate");
            document.data.remove("width");
            document.data.remove("height");
        }

        Ok(())
    }
}
