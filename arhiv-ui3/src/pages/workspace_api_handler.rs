use anyhow::{Context, Result};
use hyper::body::Bytes;

use arhiv_core::{entities::Document, markup::MarkupStr, Filter, ValidationError, Validator};

use crate::{
    app::{App, AppResponse},
    markup::MarkupStringExt,
    workspace::dto::{
        ListDocumentsResult, SaveDocumentErrors, WorkspaceRequest, WorkspaceResponse,
    },
};

impl App {
    pub fn workspace_api_handler(&self, body: &Bytes) -> Result<AppResponse> {
        let request: WorkspaceRequest =
            serde_json::from_slice(body).context("failed to parse request")?;

        let response = match request {
            WorkspaceRequest::ListDocuments { query } => {
                let mut filter = Filter::default();
                filter = filter.search(query);

                if filter.get_pattern().is_none() {
                    filter = filter.recently_updated_first();
                }

                let schema = self.arhiv.get_schema();
                let page = self.arhiv.list_documents(filter)?;

                WorkspaceResponse::ListDocuments {
                    has_more: page.has_more,
                    documents: page
                        .items
                        .into_iter()
                        .map(|item| {
                            Ok(ListDocumentsResult {
                                title: schema.get_title(&item)?,
                                id: item.id,
                                document_type: item.document_type,
                                subtype: item.subtype,
                                updated_at: item.updated_at,
                            })
                        })
                        .collect::<Result<_>>()?,
                }
            }
            WorkspaceRequest::GetStatus {} => {
                let status = self.arhiv.get_status()?;

                WorkspaceResponse::GetStatus {
                    status: status.to_string(),
                }
            }
            WorkspaceRequest::GetDocument { id } => {
                let document = self.arhiv.must_get_document(id)?;

                WorkspaceResponse::GetDocument {
                    id: document.id,
                    document_type: document.document_type,
                    subtype: document.subtype,
                    updated_at: document.updated_at,
                    data: document.data,
                }
            }
            WorkspaceRequest::RenderMarkup { markup } => {
                let markup: MarkupStr = markup.into();
                let html = markup.to_html(&self.arhiv);

                WorkspaceResponse::RenderMarkup { html }
            }
            WorkspaceRequest::GetRef { id } => {
                let document = self.arhiv.must_get_document(&id)?;
                let schema = self.arhiv.get_schema();

                WorkspaceResponse::GetRef {
                    title: schema.get_title(&document)?,
                    id,
                    document_type: document.document_type,
                    subtype: document.subtype,
                }
            }
            WorkspaceRequest::SaveDocument { id, subtype, data } => {
                let mut document = self.arhiv.must_get_document(&id)?;

                let prev_data = document.data;

                document.subtype = subtype;
                document.data = data;

                let tx = self.arhiv.get_tx()?;
                let validation_result = Validator::new(&tx).validate(&document, Some(&prev_data));

                let errors = if let Err(error) = validation_result {
                    Some(error.into())
                } else {
                    tx.stage_document(&mut document)?;

                    None
                };

                tx.commit()?;

                WorkspaceResponse::SaveDocument { errors }
            }
            WorkspaceRequest::CreateDocument {
                document_type,
                subtype,
                data,
            } => {
                let mut document = Document::new_with_data(&document_type, &subtype, data);

                let tx = self.arhiv.get_tx()?;
                let validation_result = Validator::new(&tx).validate(&document, None);

                let (id, errors) = if let Err(error) = validation_result {
                    (None, Some(error.into()))
                } else {
                    tx.stage_document(&mut document)?;

                    (Some(document.id), None)
                };

                tx.commit()?;

                WorkspaceResponse::CreateDocument { id, errors }
            }
        };

        let response = serde_json::to_string(&response).context("failed to serialize response")?;

        Ok(AppResponse::json(response))
    }
}

impl From<ValidationError> for SaveDocumentErrors {
    fn from(val: ValidationError) -> Self {
        match val {
            ValidationError::DocumentError { errors } => SaveDocumentErrors {
                document_errors: errors,
                ..Default::default()
            },
            ValidationError::FieldError { errors } => SaveDocumentErrors {
                field_errors: errors,
                ..Default::default()
            },
        }
    }
}
