use anyhow::{Context, Result};
use hyper::body::Bytes;

use arhiv_core::{markup::MarkupStr, Filter};

use crate::{
    app::{App, AppResponse},
    markup::MarkupStringExt,
    workspace::dto::{ListDocumentsResult, WorkspaceRequest, WorkspaceResponse},
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
        };

        let response = serde_json::to_string(&response).context("failed to serialize response")?;

        Ok(AppResponse::json(response))
    }
}
