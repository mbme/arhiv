use anyhow::*;
use app_shell::{ActionHandler, AppShellBuilder, AppShellContext, AppSource};
use arhiv::entities::*;
use arhiv::{markup::MarkupRenderer, markup::MarkupString, Arhiv, DocumentData, DocumentFilter};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    env_logger::init();

    let arhiv = Arc::new(Arhiv::must_open());
    let handler = Arc::new(Handler {
        arhiv: arhiv.clone(),
    });

    let src = if cfg!(feature = "production-mode") {
        AppSource::JSSource(include_str!("../dist/bundle.js").to_string())
    } else {
        AppSource::JSFile(format!("{}/dist/bundle.js", env!("CARGO_MANIFEST_DIR")))
    };

    AppShellBuilder::create("v.arhiv.ui")
        .with_title("Arhiv UI")
        .with_js_variable(
            "DATA_SCHEMA",
            serde_json::to_value(arhiv.schema.clone()).expect("must be able to convert to value"),
        )
        .start(src, handler)
        .await;
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

struct Handler {
    arhiv: Arc<Arhiv>,
}

#[async_trait]
impl ActionHandler for Handler {
    async fn run(&self, action: String, context: &AppShellContext, params: Value) -> Result<Value> {
        if action == "list" {
            let filter: DocumentFilter = serde_json::from_value(params)?;

            // FIXME validate matcher props

            let result = self.arhiv.list_documents(filter)?;

            return Ok(serde_json::to_value(&result)?);
        }

        if action == "get" {
            let id: Id = params
                .as_str()
                .context("id must be string")?
                .to_string()
                .into();

            let result = self.arhiv.get_document(&id)?;

            return Ok(serde_json::to_value(result)?);
        }

        if action == "put" {
            let mut args: PutDocumentArgs = serde_json::from_value(params)?;

            self.arhiv.schema.update_refs(&mut args.document)?;

            self.arhiv
                .stage_document(args.document, args.new_attachments)?;

            self.arhiv.sync().await?;

            return Ok(Value::Null);
        }

        if action == "create" {
            let args: CreateDocumentArgs = serde_json::from_value(params)?;

            let data = self
                .arhiv
                .schema
                .create_with_data(args.document_type.clone(), args.args)?;

            let document = Document::new(args.document_type, data.into());

            return Ok(serde_json::to_value(document)?);
        }

        if action == "get_attachment_location" {
            let id: Id = params
                .as_str()
                .context("id must be string")?
                .to_string()
                .into();

            let attachment_location = self.arhiv.get_attachment_location(&id)?;

            return Ok(serde_json::to_value(&attachment_location)?);
        }

        if action == "pick_attachments" {
            let files = context.pick_files(true);

            let attachments: Vec<AttachmentSource> = files
                .iter()
                .map(|file| AttachmentSource::new_from_path_buf(file))
                .collect();

            return Ok(serde_json::to_value(attachments)?);
        }

        if action == "render_markup" {
            let markup = params.as_str().context("markup must be string")?;

            let string = MarkupString::from(markup);

            let result =
                MarkupRenderer::new(&string, &self.arhiv, "/document".to_string()).to_html();

            return Ok(serde_json::to_value(result)?);
        }

        unreachable!("unknown action: {}", action)
    }
}
