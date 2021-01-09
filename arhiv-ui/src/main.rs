use anyhow::*;
use app_shell::{AppShellBuilder, AppSource};
use arhiv::entities::*;
use arhiv::{Arhiv, DocumentFilter};
use arhiv_modules::{
    markup::MarkupRenderer, markup::MarkupString, modules::DataSchema, modules::DocumentData,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

fn main() {
    env_logger::init();

    let arhiv = Arc::new(Arhiv::must_open());
    let data_schema = Arc::new(DataSchema::new());

    let src = if cfg!(feature = "production-mode") {
        AppSource::JSSource(include_str!("../dist/bundle.js").to_string())
    } else {
        AppSource::JSFile(format!("{}/dist/bundle.js", env!("CARGO_MANIFEST_DIR")))
    };

    AppShellBuilder::create("v.arhiv.ui")
        .with_title("Arhiv UI")
        .with_js_variable("DATA_SCHEMA", DataSchema::SCHEMA)
        .with_action("list", {
            let arhiv = arhiv.clone();

            move |_, params| {
                let filter: DocumentFilter = serde_json::from_value(params)?;

                // FIXME validate matcher props

                let result = arhiv.list_documents(filter)?;

                Ok(serde_json::to_value(&result)?)
            }
        })
        .with_action("get", {
            let arhiv = arhiv.clone();

            move |_, params| {
                let id: Id = params
                    .as_str()
                    .context("id must be string")?
                    .to_string()
                    .into();

                let result = arhiv.get_document(&id)?;

                Ok(serde_json::to_value(result)?)
            }
        })
        .with_action("put", {
            let arhiv = arhiv.clone();
            let data_schema = data_schema.clone();

            move |_, params| {
                let mut args: PutDocumentArgs = serde_json::from_value(params)?;

                data_schema.update_refs(&mut args.document)?;

                arhiv.stage_document(args.document, args.new_attachments)?;

                Ok(Value::Null)
            }
        })
        .with_action("create", {
            let data_schema = data_schema.clone();
            move |_, params| {
                let args: CreateDocumentArgs = serde_json::from_value(params)?;

                let data = data_schema.create_with_data(args.document_type.clone(), args.args)?;

                let document = Document::new(args.document_type, data.into());

                Ok(serde_json::to_value(document)?)
            }
        })
        .with_action("get_attachment_location", {
            let arhiv = arhiv.clone();

            move |_, params| {
                let id: Id = params
                    .as_str()
                    .context("id must be string")?
                    .to_string()
                    .into();

                let attachment_location = arhiv.get_attachment_location(&id)?;

                Ok(serde_json::to_value(&attachment_location)?)
            }
        })
        .with_action("pick_attachments", {
            move |context, _params| {
                let files = context.pick_files(true);

                let attachments: Vec<AttachmentSource> = files
                    .iter()
                    .map(|file| AttachmentSource::new_from_path_buf(file))
                    .collect();

                Ok(serde_json::to_value(attachments)?)
            }
        })
        .with_action("render_markup", {
            let arhiv = arhiv.clone();

            move |_, params| {
                let markup = params.as_str().context("markup must be string")?;

                let string = MarkupString::from(markup);

                let result =
                    MarkupRenderer::new(&string, &arhiv, "/document".to_string()).to_html();

                Ok(serde_json::to_value(result)?)
            }
        })
        .start(src);
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateDocumentArgs {
    document_type: String,
    args: DocumentData,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PutDocumentArgs {
    document: Document,
    new_attachments: Vec<AttachmentSource>,
}
