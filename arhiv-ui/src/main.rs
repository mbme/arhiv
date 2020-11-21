use app_shell::{AppShellBuilder, AppSource};
use arhiv::entities::*;
use arhiv::{Arhiv, DocumentFilter};
use arhiv_modules::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

fn main() {
    env_logger::init();

    let arhiv = Arc::new(Arhiv::must_open());

    let src = if cfg!(feature = "production-mode") {
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
                let args: PutDocumentArgs =
                    serde_json::from_value(params).expect("failed to parse params");

                let mut document = match args.document.document_type.as_str() {
                    Note::TYPE => Note::from_document(args.document),
                    _ => {
                        log::error!(
                            "action put: got document of unknown type {}",
                            &args.document.document_type
                        );

                        return Value::Null;
                    }
                };

                // Extract & update refs
                document.0.refs = document.extract_refs();

                arhiv
                    .stage_document(document.into_document(), args.new_attachments)
                    .expect("must be able to save document");

                Value::Null
            }
        })
        .with_action("create", {
            move |_, params| {
                let args: CreateDocumentArgs =
                    serde_json::from_value(params).expect("failed to parse params");

                match args.document_type.as_str() {
                    Note::TYPE => {
                        //
                        serde_json::to_value(Note::new().into_document())
                            .expect("must be able to serialize")
                    }
                    Project::TYPE => {
                        //
                        serde_json::to_value(Project::new().into_document())
                            .expect("must be able to serialize")
                    }
                    Task::TYPE => {
                        //
                        let project_id: Id =
                            serde_json::from_value(args.args).expect("failed to parse id");

                        serde_json::to_value(Task::new(project_id).into_document())
                            .expect("must be able to serialize")
                    }
                    _ => {
                        log::error!("action create: got unknown type {}", args.document_type);

                        Value::Null
                    }
                }
            }
        })
        .with_action("parse_markup", {
            move |_, params| {
                let markup = params.as_str().expect("markup must be string");

                let result = parse_markup(markup);

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
            move |context, _params| {
                let files = context.pick_files(true);

                let attachments: Vec<AttachmentSource> = files
                    .iter()
                    .map(|file| AttachmentSource::new_from_path_buf(file))
                    .collect();

                serde_json::to_value(attachments).expect("must be able to serialize")
            }
        })
        .start(src);
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateDocumentArgs {
    document_type: String,
    args: Value,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PutDocumentArgs {
    document: Document,
    new_attachments: Vec<AttachmentSource>,
}
