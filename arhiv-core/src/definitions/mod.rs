use crate::schema::DataSchema;

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
        vec![],
    )
}
