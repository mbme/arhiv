use anyhow::*;
use app_shell::{AppShellBuilder, AppSource};
use arhiv::entities::*;
use arhiv::{Arhiv, DocumentFilter};
use arhiv_modules::{
    markup::MarkupRenderer, markup::MarkupString, modules::DocumentData,
    modules::DocumentDataManager,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

fn main() {
    env_logger::init();

    let arhiv = Arc::new(Arhiv::must_open());
    let data_manager = Arc::new(DocumentDataManager::new());

    let src = if cfg!(feature = "production-mode") {
        AppSource::JSSource(include_str!("../dist/bundle.js").to_string())
    } else {
        AppSource::JSFile(format!("{}/dist/bundle.js", env!("CARGO_MANIFEST_DIR")))
    };

    AppShellBuilder::create("v.arhiv.ui")
        .with_title("Arhiv UI")
        .with_js_variable(
            "DATA_DESCRIPTION",
            serde_json::to_value(&data_manager.modules).expect("must be able to convert to value"),
        )
        .with_action("list", {
            let arhiv = arhiv.clone();

            move |_, params| {
                let filter: Option<DocumentFilter> =
                    serde_json::from_value(params).context("param must be document filter")?;

                // FIXME validate matcher props

                let result = arhiv
                    .list_documents(filter)
                    .context("must be able to list documents")?;

                serde_json::to_value(&result).context("must be able to serialize")
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

                let result = arhiv
                    .get_document(&id)
                    .context("must be able to get document")?;

                serde_json::to_value(result).context("must be able to serialize")
            }
        })
        .with_action("put", {
            let arhiv = arhiv.clone();
            let data_manager = data_manager.clone();

            move |_, params| {
                let mut args: PutDocumentArgs =
                    serde_json::from_value(params).context("failed to parse params")?;

                data_manager
                    .update_refs(&mut args.document)
                    .context("must be able to update refs")?;

                arhiv
                    .stage_document(args.document, args.new_attachments)
                    .context("must be able to save document")?;

                Ok(Value::Null)
            }
        })
        .with_action("create", {
            let data_manager = data_manager.clone();
            move |_, params| {
                let args: CreateDocumentArgs =
                    serde_json::from_value(params).context("failed to parse params")?;

                let data = data_manager
                    .create_with_data(args.document_type, args.args)
                    .context("must be able to create document data")?;
                let document = Document::new(data.into());

                serde_json::to_value(document).context("must be able to serialize")
            }
        })
        .with_action("get_attachment", {
            let arhiv = arhiv.clone();

            move |_, params| {
                let id: Id = params
                    .as_str()
                    .context("id must be string")?
                    .to_string()
                    .into();

                let attachment = arhiv
                    .get_attachment(&id)
                    .context("must be able to get attachment")?;

                serde_json::to_value(&attachment).context("must be able to serialize")
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

                let attachment_location = arhiv
                    .get_attachment_location(&id)
                    .context("must be able to get attachment location")?;

                serde_json::to_value(&attachment_location).context("must be able to serialize")
            }
        })
        .with_action("pick_attachments", {
            move |context, _params| {
                let files = context.pick_files(true);

                let attachments: Vec<AttachmentSource> = files
                    .iter()
                    .map(|file| AttachmentSource::new_from_path_buf(file))
                    .collect();

                serde_json::to_value(attachments).context("must be able to serialize")
            }
        })
        .with_action("render_markup", {
            let arhiv = arhiv.clone();

            move |_, params| {
                let markup = params.as_str().context("markup must be string")?;

                let string = MarkupString::from(markup);

                let result =
                    MarkupRenderer::new(&string, &arhiv, "/document".to_string()).to_html();

                serde_json::to_value(result).context("must be able to serialize")
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
