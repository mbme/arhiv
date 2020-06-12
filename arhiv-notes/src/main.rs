use app_shell::AppShellBuilder;
use arhiv::entities::Document;
use arhiv_notes::notes::ArhivNotes;
use serde_json::Value;
use std::rc::Rc;

fn main() {
    env_logger::init();

    let path_str = format!("{}/static/index.html", env!("CARGO_MANIFEST_DIR"));

    let notes = Rc::new(ArhivNotes::must_open());

    AppShellBuilder::create("v.arhiv.notes")
        .with_title("Arhiv Notes")
        .with_action("list", {
            let notes = notes.clone();

            move |_params| serde_json::to_value(&notes.list()).expect("must be able to serialize")
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
        .show_inspector()
        .load(path_str);
}
