use arhiv::arhiv::Arhiv;
use arhiv::config::Config;
use arhiv::entities::*;
use std::collections::HashMap;

fn gen_random_document() -> Document {
    let mut document = Document::new("note");

    document.data = "{ name: \"test\", data: \"data\" }".to_string();

    document
}

fn main() {
    env_logger::init();

    let arhiv = Arhiv::open(Config::read().unwrap()).expect("must be able to open arhiv");

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
