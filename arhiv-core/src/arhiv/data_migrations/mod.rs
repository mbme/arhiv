use std::{
    panic::{RefUnwindSafe, UnwindSafe},
    sync::Arc,
};

use anyhow::Result;

use crate::entities::DocumentData;

pub trait DataMigration: Send + Sync + UnwindSafe + RefUnwindSafe {
    fn get_version(&self) -> u8;

    fn update(&self, document_type: &str, data: &mut DocumentData) -> Result<()>;
}

#[must_use]
pub fn get_migrations() -> Vec<Arc<dyn DataMigration>> {
    vec![
        //
        Arc::new(DataSchema1),
    ]
}

#[must_use]
pub fn get_version() -> u8 {
    get_migrations()
        .iter()
        .fold(0, |latest_version, migration| {
            migration.get_version().max(latest_version)
        })
}

struct DataSchema1;

impl DataMigration for DataSchema1 {
    fn get_version(&self) -> u8 {
        1
    }

    fn update(&self, document_type: &str, data: &mut DocumentData) -> Result<()> {
        // replace "completed" with "status"
        if let Some(completed) = data.get_bool("completed") {
            if completed {
                data.set("status", "Completed");
            }
            data.remove("completed");
        }

        // in film
        // remove is_series
        // rename episode_duration -> duration
        // rename number_of_episodes -> episodes
        // rename number_of_seasons -> seasons
        if document_type == "film" {
            data.remove("is_series");
            data.rename("episode_duration", "duration");
            data.rename("number_of_episodes", "episodes");
            data.rename("number_of_seasons", "seasons");
        }

        Ok(())
    }
}
