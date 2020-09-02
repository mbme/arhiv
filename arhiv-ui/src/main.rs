use app_shell::{pick_files, AppShellBuilder, AppSource};
use arhiv::entities::*;
use arhiv::Arhiv;
use arhiv_modules::ArhivNotes;
use serde_json::Value;
use std::rc::Rc;

fn main() {
    env_logger::init();

    let notes = Rc::new(ArhivNotes::new(Arhiv::must_open()));

    AppShellBuilder::create("v.arhiv.notes")
        .with_title("Arhiv Notes")
        .with_action("list", {
            let notes = notes.clone();

            move |params| {
                let pattern = params.as_str().expect("id must be string").to_string();

                serde_json::to_value(&notes.list(pattern)).expect("must be able to serialize")
            }
        })
        .with_action("get_note", {
            let notes = notes.clone();

            move |params| {
                let id = params.as_str().expect("id must be string").to_string();

                serde_json::to_value(notes.get_note(&id)).expect("must be able to serialize")
            }
        })
        .with_action("put_note", {
            let notes = notes.clone();

            move |params| {
                let note: Document =
                    serde_json::from_value(params).expect("param must be document");

                notes.put_note(note);

                Value::Null
            }
        })
        .with_action("create_note", {
            move |_params| {
                serde_json::to_value(ArhivNotes::create_note()).expect("must be able to serialize")
            }
        })
        .with_action("get_attachment", {
            let notes = notes.clone();

            move |params| {
                let id = params.as_str().expect("id must be string").to_string();

                serde_json::to_value(notes.arhiv.get_attachment(&id).unwrap())
                    .expect("must be able to serialize")
            }
        })
        .with_action("get_attachment_location", {
            let notes = notes.clone();

            move |params| {
                let id = params.as_str().expect("id must be string").to_string();

                serde_json::to_value(notes.arhiv.get_attachment_location(id).unwrap())
                    .expect("must be able to serialize")
            }
        })
        .with_action("pick_attachments", {
            let notes = notes.clone();

            move |_params| {
                let files = pick_files(true);

                let attachments: Vec<Attachment> = files
                    .iter()
                    .map(|file| {
                        notes
                            .arhiv
                            .stage_attachment(file.to_str().unwrap(), false)
                            .unwrap()
                    })
                    .collect();

                serde_json::to_value(attachments).expect("must be able to serialize")
            }
        })
        .show_inspector()
        .load(AppSource::JSFile(format!(
            "{}/dist/bundle.js",
            env!("CARGO_MANIFEST_DIR")
        )));
}
