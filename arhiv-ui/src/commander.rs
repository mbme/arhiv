use anyhow::*;
use arhiv::{entities::*, markup::*, schema::DocumentData, Arhiv, Filter};
use rs_utils::run_command;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct ArhivCommander {
    arhiv: Arhiv,
}

impl ArhivCommander {
    pub fn new(arhiv: Arhiv) -> ArhivCommander {
        ArhivCommander { arhiv }
    }

    pub async fn run(&self, action: String, params: Value) -> Result<Value> {
        if action == "get_schema" {
            return Ok(serde_json::to_value(self.arhiv.schema.clone())?);
        }

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
