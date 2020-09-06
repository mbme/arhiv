use arhiv::entities::*;
use arhiv::Arhiv;
use arhiv_modules::{create_link, ArhivNotes};
use rand::prelude::*;
use rand::thread_rng;
use rs_utils::{project_relpath, Markov};
use serde_json::json;
use std::fs;

fn gen_note(markov: &Markov, attachment_ids: &Vec<Id>) -> Document {
    let name = markov.generate_sentence_constrained(8, false);

    // generate note data
    let mut data: Vec<String> = vec![];
    let mut rng = thread_rng();
    for _ in 0..rng.gen_range(1, 8) {
        let mut sentences = vec![];

        for _ in 0..rng.gen_range(1, 8) {
            sentences.push(markov.generate_sentence_constrained(20, true));
        }

        if rng.gen_bool(0.33) {
            let id = attachment_ids
                .choose(&mut rng)
                .expect("attachment ids must be provided");

            sentences.push(create_link(id, id));
        }

        sentences.shuffle(&mut rng);

        data.push(sentences.join(" "));
    }

    let mut document = ArhivNotes::create_note();
    document.data = json!({ "name": name, "data": data.join("\n\n") });
    document.attachment_refs = attachment_ids.clone();

    document
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let notes = ArhivNotes::new(Arhiv::must_open());

    let mut attachment_ids: Vec<Id> = vec![];

    let dir = project_relpath("../resources");
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let path = path.to_str().unwrap();

        if path.ends_with(".jpg") || path.ends_with(".jpeg") {
            let attachment = notes
                .arhiv
                .stage_attachment(path, false)
                .expect("must be able to create attachment");
            attachment_ids.push(attachment.id);
        }
    }

    let text = fs::read_to_string(project_relpath("../resources/text.txt")).unwrap();
    let markov = Markov::new(&text);

    for _ in 0..30 {
        notes.put_note(gen_note(&markov, &attachment_ids));
    }

    notes.arhiv.sync().await.expect("must be able to sync");
}
