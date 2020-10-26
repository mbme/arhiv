use app_shell::{AppShellBuilder, AppSource};
use arhiv::entities::*;
use arhiv::{Arhiv, DocumentFilter};
use arhiv_modules::*;
use rs_utils::is_production_mode;
use serde_json::Value;
use std::sync::Arc;

fn main() {
    env_logger::init();

    let arhiv = Arc::new(Arhiv::must_open());

    let src = if is_production_mode() {
        AppSource::JSSource(include_str!("../dist/bundle.js").to_string())
    } else {
        AppSource::JSFile(format!("{}/dist/bundle.js", env!("CARGO_MANIFEST_DIR")))
    };

    AppShellBuilder::create("v.arhiv.ui")
        .with_title("Arhiv UI")
        .with_action("list", {
            let arhiv = arhiv.clone();

            move |_, params| {
                let filter: Option<DocumentFilter> =
                    serde_json::from_value(params).expect("param must be document filter");

                let result = arhiv
                    .list_documents(filter)
                    .expect("must be able to list documents");

                serde_json::to_value(&result).expect("must be able to serialize")
            }
        })
        .with_action("get", {
            let arhiv = arhiv.clone();

            move |_, params| {
                let id: Id = params
                    .as_str()
                    .expect("id must be string")
                    .to_string()
                    .into();

                let result = arhiv
                    .get_document(&id)
                    .expect("must be able to get document");

                serde_json::to_value(result).expect("must be able to serialize")
            }
        })
        .with_action("put", {
            let arhiv = arhiv.clone();

            move |_, params| {
                let document: Document =
                    serde_json::from_value(params).expect("param must be document");

                arhiv
                    .stage_document(document)
                    .expect("must be able to save document");

                Value::Null
            }
        })
        .with_action("create", {
            move |_, params| {
                let document_type = params.as_str().expect("type must be string");

                let result = match document_type {
                    Note::TYPE => Some(Note::new().into_document()),
                    _ => {
                        log::error!("action create: got unknown type {}", document_type);

                        None
                    }
                };

                serde_json::to_value(result).expect("must be able to serialize")
            }
        })
        .with_action("get_attachment", {
            let arhiv = arhiv.clone();

            move |_, params| {
                let id: Id = params
                    .as_str()
                    .expect("id must be string")
                    .to_string()
                    .into();

                serde_json::to_value(arhiv.get_attachment(&id).unwrap())
                    .expect("must be able to serialize")
            }
        })
        .with_action("get_attachment_location", {
            let arhiv = arhiv.clone();

            move |_, params| {
                let id: Id = params
                    .as_str()
                    .expect("id must be string")
                    .to_string()
                    .into();

                serde_json::to_value(arhiv.get_attachment_location(id).unwrap())
                    .expect("must be able to serialize")
            }
        })
        .with_action("pick_attachments", {
            let arhiv = arhiv.clone();

            move |context, _params| {
                let files = context.pick_files(true);

                let attachments: Vec<Attachment> = files
                    .iter()
                    .map(|file| {
                        arhiv
                            .stage_attachment(file.to_str().unwrap(), false)
                            .unwrap()
                    })
                    .collect();

                serde_json::to_value(attachments).expect("must be able to serialize")
            }
        })
        .start(src);
}
