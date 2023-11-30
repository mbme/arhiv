use std::borrow::Cow;

use anyhow::{Context, Result};

use baza::{entities::Document, schema::DataMigration, BazaConnection};

pub struct DataSchema5;

/// Remove some task statuses
impl DataMigration for DataSchema5 {
    fn get_version(&self) -> u8 {
        5
    }

    fn update(&self, document: &mut Cow<Document>, _conn: &BazaConnection) -> Result<()> {
        if document.class.document_type == "task" {
            let status = document
                .data
                .get_str("status")
                .context("status must be present")?;

            if status == "Inbox" || status == "Later" {
                document.to_mut().data.set("status", "Todo");
            } else if status == "Paused" {
                document.to_mut().data.set("status", "InProgress");
            }
        }

        Ok(())
    }
}
