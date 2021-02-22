use anyhow::*;
use app_shell::{ActionHandler, AppShellBuilder, AppSource};
use arhiv::{entities::*, markup::RenderOptions, schema::DocumentData};
use arhiv::{markup::MarkupRenderer, markup::MarkupString, Arhiv, Filter};
use async_trait::async_trait;
use rs_utils::{
    log::{setup_logger, warn},
    run_command,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    setup_logger();

    let arhiv = Arc::new(Arhiv::must_open());
    let handler = Arc::new(Handler {
        arhiv: arhiv.clone(),
    });

    let src = if cfg!(feature = "production-mode") {
        AppSource::JSSource(include_str!("../dist/bundle.js").to_string())
    } else {
        AppSource::JSFile(format!("{}/dist/bundle.js", env!("CARGO_MANIFEST_DIR")))
    };

    let app_future = AppShellBuilder::create("v.arhiv.ui")
        .with_title("Arhiv UI")
        .with_js_variable(
            "DATA_SCHEMA",
            serde_json::to_value(arhiv.schema.clone()).expect("must be able to convert to value"),
        )
        .start(src, handler);

    let sync_future = async {
        if let Err(err) = arhiv.sync().await {
            warn!("initial sync failed: {}", err);
        }
    };

    tokio::join!(sync_future, app_future);
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
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RenderMarkupArgs {
    value: String,
    options: RenderOptions,
}

struct Handler {
    arhiv: Arc<Arhiv>,
}

#[async_trait]
impl ActionHandler for Handler {
    async fn run(&self, action: String, params: Value) -> Result<Value> {
        if action == "list" {
            let filter: Filter = serde_json::from_value(params)?;

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

            self.arhiv.stage_document(args.document)?;

            self.arhiv.sync().await?;

            return Ok(Value::Null);
        }

        if action == "create" {
            let args: CreateDocumentArgs = serde_json::from_value(params)?;

            let data = self
                .arhiv
                .schema
                .create_with_initial_values(args.document_type.clone(), args.args)?;

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

        if action == "get_status" {
            let status = self.arhiv.get_status()?;

            return Ok(serde_json::to_value(&status.to_string())?);
        }

        if action == "pick_attachments" {
            let files = run_command("mb-filepicker", vec!["-m"])?;
            let files: Vec<String> = serde_json::from_str(&files)?;

            let mut attachment_ids: Vec<Id> = vec![];
            for file_path in files {
                let document = self.arhiv.add_attachment(file_path, false)?;
                attachment_ids.push(document.id);
            }

            return Ok(serde_json::to_value(attachment_ids)?);
        }

        if action == "render_markup" {
            let args: RenderMarkupArgs = serde_json::from_value(params)?;

            let string = MarkupString::from(args.value);

            let result = MarkupRenderer::new(&self.arhiv, &args.options).to_html(&string);

            return Ok(serde_json::to_value(result)?);
        }

        unreachable!("unknown action: {}", action)
    }
}
