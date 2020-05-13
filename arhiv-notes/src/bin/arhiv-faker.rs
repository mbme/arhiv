use arhiv::entities::*;
use arhiv::Arhiv;
use arhiv_notes::notes::NOTE_TYPE;
use std::collections::HashMap;

fn gen_random_document() -> Document {
    let mut document = Document::new(NOTE_TYPE);

    document.data = "{ name: \"test\", data: \"data\" }".to_string();

    document
}

fn main() {
    env_logger::init();

    let arhiv = Arhiv::must_open();

    let mut documents = vec![];

    for _ in 0..30 {
        let document = gen_random_document();
        documents.push(document);
    }

    let changeset = Changeset {
        base_rev: 0,
        documents,
        attachments: vec![],
    };

    arhiv
        .apply_changeset(changeset, HashMap::new())
        .expect("must be able to apply changeset with fake data");
}
