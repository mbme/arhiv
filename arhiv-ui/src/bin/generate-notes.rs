use arhiv::entities::*;
use arhiv::{Arhiv, ArhivNotes};
use binutils::utils::run_command;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct FakerArgs {
    text: String,
    attachment_ids: Vec<Id>,
    count: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct NoteData {
    name: String,
    data: String,
}

fn gen_notes(text: String, count: u8, attachment_ids: Vec<Id>) -> Vec<NoteData> {
    let arg = FakerArgs {
        text,
        attachment_ids,
        count,
    };
    let arg = serde_json::to_string(&arg).expect("must be able to serialize args");

    let result = run_command(
        "yarn",
        vec![
            "run",
            "-s",
            "ts-node",
            "--transpile-only",
            "-O",
            "{ \"module\": \"commonjs\" }",
            "scripts/faker.ts",
            &arg,
        ],
    )
    .expect("must be able to run script");

    let result: Vec<NoteData> =
        serde_json::from_str(&result).expect("must be able to parse response");

    result
}

fn relpath(subpath: &str) -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), subpath)
}

fn main() {
    env_logger::init();

    let notes = Arhiv::must_open().notes();

    let mut attachment_ids: Vec<Id> = vec![];

    let dir = relpath("resources");
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let path = path.to_str().unwrap();

        if path.ends_with(".jpg") || path.ends_with(".jpeg") {
            let attachment = notes
                .arhiv
                .stage_attachment(path)
                .expect("must be able to create attachment");
            attachment_ids.push(attachment.id);
        }
    }

    let text = fs::read_to_string(relpath("resources/text.txt")).unwrap();
    for note_data in gen_notes(text, 30, attachment_ids.clone()) {
        let mut document = ArhivNotes::create_note();
        document.data = json!({ "name": note_data.name, "data": note_data.data });
        document.attachment_refs = attachment_ids.clone();
        notes.put_note(document);
    }

    notes.arhiv.commit().expect("must be able to commit");
}
