use crate::schema::DataSchema;

mod book;
mod fields;
mod film;
mod note;
mod task;

pub fn get_schema() -> DataSchema {
    let mut schema = DataSchema::new();
    schema.modules.append(
        &mut vec![
            note::get_note_definitions(),
            task::get_task_definitions(),
            book::get_book_definitions(),
            film::get_film_definitions(),
        ]
        .concat(),
    );

    schema
}
