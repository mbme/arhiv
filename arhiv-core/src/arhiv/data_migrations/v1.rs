use anyhow::Result;

use crate::entities::Document;

use super::migration::DataMigration;

pub struct DataSchema1;

impl DataMigration for DataSchema1 {
    fn get_version(&self) -> u8 {
        1
    }

    // TODO: use Cow
    fn update(&self, document: &mut Document) -> Result<()> {
        // replace "completed" with "status"
        if let Some(completed) = document.data.get_bool("completed") {
            if completed {
                document.data.set("status", "Completed");
            }
            document.data.remove("completed");
        }

        // in film
        // remove is_series
        // rename episode_duration -> duration
        // rename number_of_episodes -> episodes
        // rename number_of_seasons -> seasons
        if document.document_type == "film" {
            document.data.remove("is_series");
            document.data.rename("episode_duration", "duration");
            document.data.rename("number_of_episodes", "episodes");
            document.data.rename("number_of_seasons", "seasons");
        }

        Ok(())
    }
}
