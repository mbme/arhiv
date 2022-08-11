use anyhow::{Context, Result};
use hyper::body::Bytes;

use arhiv_core::Filter;

use crate::{
    app::{App, AppResponse},
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
                let data_description = self
                    .arhiv
                    .get_schema()
                    .get_data_description(&document.document_type)?;

                WorkspaceResponse::GetDocument {
                    id: document.id,
                    document_type: document.document_type,
                    subtype: document.subtype,
                    updated_at: document.updated_at,
                    data: document.data,
                    data_description,
                }
            }
        };

        let response = serde_json::to_string(&response).context("failed to serialize response")?;

        Ok(AppResponse::json(response))
    }
}
