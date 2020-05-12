use app_shell::AppShellBuilder;
use arhiv::entities::Document;
use notes::ArhivNotes;
use serde_json::Value;
use std::rc::Rc;

mod notes;

fn main() {
    env_logger::init();

    let path_str = format!("{}/static/index.html", env!("CARGO_MANIFEST_DIR"));

    let notes = ArhivNotes::must_open();
    let action_handler = Rc::new(move |action: String, params: Value| {
        if action == "list" {
            return serde_json::to_value(notes.list()).expect("must be able to serialize");
        }

        if action == "get_note" {
            let id = params.as_str().expect("id must be string").to_string();

            return serde_json::to_value(notes.get_note(&id)).expect("must be able to serialize");
        }

        if action == "put_note" {
            let note: Document = serde_json::from_value(params).expect("param must be document");

            notes.put_note(note);

            return Value::Null;
        }

        Value::Null
    });

    AppShellBuilder::create("v.arhiv.notes".to_string())
        .with_title("Arhiv Notes".to_string())
        .with_rpc(action_handler)
        .show_inspector()
        .load(path_str);
}
