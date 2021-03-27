use std::sync::Arc;

use crate::{
    entities::*,
    markup::*,
    schema::{DocumentData, FieldType, SCHEMA},
    Arhiv, Filter,
};
use anyhow::*;
use rs_utils::run_command;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct ArhivCommander {
    arhiv: Arc<Arhiv>,
    render_options: RenderOptions,
}

impl ArhivCommander {
    pub fn new(arhiv: Arc<Arhiv>, render_options: RenderOptions) -> ArhivCommander {
        ArhivCommander {
            arhiv,
            render_options,
        }
    }

    fn get_renderer(&self) -> MarkupRenderer {
        MarkupRenderer::new(&self.arhiv, &self.render_options)
    }

    fn get_preview(&self, document: &Document) -> Result<String> {
        let field = SCHEMA.pick_title_field(&document.document_type)?;

        match field.field_type {
            FieldType::MarkupString {} => {
                let text = SCHEMA.get_field_string(document, field.name)?;
                let markup: MarkupString = text.lines().take(4).collect::<String>().into();

                Ok(self.get_renderer().to_html(&markup))
            }
            FieldType::String {} => SCHEMA.get_field_string(document, field.name),
            _ => unimplemented!(),
        }
    }

    pub async fn run(&self, action: String, params: Value) -> Result<Value> {
        if action == "get_schema" {
            return Ok(serde_json::to_value(SCHEMA.clone())?);
        }

        if action == "list" {
            let filter: Filter = serde_json::from_value(params)?;

            // FIXME validate matcher props

            let result = self
                .arhiv
                .list_documents(filter)?
                .map(|document| DocumentExt {
                    preview: self
                        .get_preview(&document)
                        .unwrap_or("No preview".to_string()),
                    document,
                });

            return Ok(serde_json::to_value(&result)?);
        }

        if action == "get" {
            let id = Id::from_string(params.as_str().context("id must be string")?.to_string());

            let result = self.arhiv.get_document(&id)?;

            return Ok(serde_json::to_value(result)?);
        }

        if action == "put" {
            let mut args: PutDocumentArgs = serde_json::from_value(params)?;

            SCHEMA.update_refs(&mut args.document)?;

            self.arhiv.stage_document(args.document)?;

            return Ok(Value::Null);
        }

        if action == "create" {
            let args: CreateDocumentArgs = serde_json::from_value(params)?;

            let data = SCHEMA.create_with_initial_values(args.document_type.clone(), args.args)?;

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
                let document = self.arhiv.add_attachment(&file_path, false)?;
                attachment_ids.push(document.id);
            }

            return Ok(serde_json::to_value(attachment_ids)?);
        }

        if action == "render_markup" {
            let string = MarkupString::from(
                params
                    .as_str()
                    .context("markup must be string")?
                    .to_string(),
            );

            let result = self.get_renderer().to_html(&string);

            return Ok(serde_json::to_value(result)?);
        }

        if action == "sync" {
            self.arhiv.sync().await?;

            return Ok(Value::Null);
        }

        if action == "is_sync_required" {
            let is_sync_required = self.arhiv.get_status()?.is_sync_required();

            return Ok(serde_json::to_value(is_sync_required)?);
        }

        unreachable!("unknown action: {}", action)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateDocumentArgs {
    document_type: String,
    args: DocumentData,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PutDocumentArgs {
    document: Document,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentExt {
    document: Document,
    preview: String,
}
