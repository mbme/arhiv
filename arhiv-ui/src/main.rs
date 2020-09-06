use app_shell::{AppShellBuilder, AppSource};
use arhiv::entities::*;
use arhiv::Arhiv;
use arhiv_modules::ArhivNotes;
use rs_utils::is_production_mode;
use serde_json::Value;
use std::sync::Arc;

fn main() {
    env_logger::init();

    let notes = Arc::new(ArhivNotes::new(Arhiv::must_open()));

    let src = if is_production_mode() {
        AppSource::JSSource(include_str!("../dist/bundle.js").to_string())
    } else {
        AppSource::JSFile(format!("{}/dist/bundle.js", env!("CARGO_MANIFEST_DIR")))
    };

    AppShellBuilder::create("v.arhiv.ui")
        .with_title("Arhiv UI")
        .with_action("list", {
            let notes = notes.clone();

            move |_, params| {
                let pattern = params.as_str().expect("id must be string").to_string();

                serde_json::to_value(&notes.list(pattern)).expect("must be able to serialize")
            }
        })
        .with_action("get_note", {
            let notes = notes.clone();

            move |_, params| {
                let id = params.as_str().expect("id must be string").to_string();

                serde_json::to_value(notes.get_note(&id)).expect("must be able to serialize")
            }
        })
        .with_action("put_note", {
            let notes = notes.clone();

            move |_, params| {
                let note: Document =
                    serde_json::from_value(params).expect("param must be document");

                notes.put_note(note);

                Value::Null
            }
        })
        .with_action("create_note", {
            move |_, _params| {
                serde_json::to_value(ArhivNotes::create_note()).expect("must be able to serialize")
            }
        })
        .with_action("get_attachment", {
            let notes = notes.clone();

            move |_, params| {
                let id = params.as_str().expect("id must be string").to_string();

                serde_json::to_value(notes.arhiv.get_attachment(&id).unwrap())
                    .expect("must be able to serialize")
            }
        })
        .with_action("get_attachment_location", {
            let notes = notes.clone();

            move |_, params| {
                let id = params.as_str().expect("id must be string").to_string();

                serde_json::to_value(notes.arhiv.get_attachment_location(id).unwrap())
                    .expect("must be able to serialize")
            }
        })
        .with_action("pick_attachments", {
            let notes = notes.clone();

            move |context, _params| {
                let files = context.pick_files(true);

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
        .load(src);
}
