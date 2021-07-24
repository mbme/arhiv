use crate::schema::DataDescription;

mod book;
mod fields;
mod film;
mod note;
mod task;

pub fn get_definitions() -> Vec<DataDescription> {
    vec![
        note::get_note_definitions(),
        task::get_task_definitions(),
        book::get_book_definitions(),
        film::get_film_definitions(),
    ]
    .concat()
}
