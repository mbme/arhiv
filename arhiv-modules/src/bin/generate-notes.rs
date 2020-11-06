use arhiv::Arhiv;
use arhiv_modules::generator::*;
use arhiv_modules::*;

#[tokio::main]
async fn main() {
    env_logger::init();

    let arhiv = Arhiv::must_open();
    let attachment_ids = create_attachments(&arhiv);
    let generator = Generator::new(attachment_ids);

    for _ in 0..30 {
        let mut note = Note::new();

        note.0.data = NoteData {
            name: generator.gen_string(),
            data: generator.gen_markup_string(1, 8),
        };

        arhiv
            .stage_document(note.into_document())
            .expect("must be able to save document");
    }

    arhiv.sync().await.expect("must be able to sync");
}
