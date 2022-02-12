use std::sync::Arc;

use anyhow::Result;

use crate::{
    entities::DocumentData,
    schema::{DataSchema, SchemaMigration},
};

pub use attachment::{Attachment, ATTACHMENT_TYPE};
pub use book::{BOOK_COLLECTION_TYPE, BOOK_TYPE};
pub use contact::{CONTACT_COLLECTION_TYPE, CONTACT_TYPE};
pub use film::{FILM_COLLECTION_TYPE, FILM_TYPE};
pub use game::{GAME_COLLECTION_TYPE, GAME_TYPE};
pub use note::NOTE_TYPE;
pub use task::{PROJECT_TYPE, TASK_STATUS, TASK_TYPE};
pub use track::{TRACK_COLLECTION_TYPE, TRACK_TYPE};

mod fields;

mod attachment;
mod book;
mod contact;
mod film;
mod game;
mod note;
mod task;
mod track;

#[must_use]
pub fn get_standard_schema() -> DataSchema {
    DataSchema::new(
        vec![
            attachment::get_attachment_definitions(),
            note::get_note_definitions(),
            task::get_task_definitions(),
            book::get_book_definitions(),
            film::get_film_definitions(),
            game::get_game_definitions(),
            track::get_track_definitions(),
            contact::get_contact_definitions(),
        ]
        .concat(),
        vec![
            //
            Arc::new(Schema1),
        ],
    )
}

struct Schema1;

impl SchemaMigration for Schema1 {
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
