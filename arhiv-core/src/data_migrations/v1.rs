use std::borrow::Cow;

use anyhow::Result;

use baza::{entities::Document, schema::DataMigration, BazaConnection};

pub struct DataSchema1;

impl DataMigration for DataSchema1 {
    fn get_version(&self) -> u8 {
        1
    }

    fn update(&self, document: &mut Cow<Document>, _conn: &BazaConnection) -> Result<()> {
        // replace "completed" with "status"
        if let Some(completed) = document.data.get_bool("completed") {
            if completed {
                document.to_mut().data.set("status", "Completed");
            }
            document.to_mut().data.remove("completed");
        }

        // in film
        // remove is_series
        // rename episode_duration -> duration
        // rename number_of_episodes -> episodes
        // rename number_of_seasons -> seasons
        if document.document_type.document_type == "film" {
            let data = &mut document.to_mut().data;

            data.remove("is_series");
            data.rename("episode_duration", "duration");
            data.rename("number_of_episodes", "episodes");
            data.rename("number_of_seasons", "seasons");
        }

        Ok(())
    }
}
