use anyhow::{Context, Result};
use hyper::body::Bytes;

use arhiv_core::Filter;

use crate::{
    app::{App, AppResponse},
    workspace::dto::{WorkspaceRequest, WorkspaceResponse},
};

impl App {
    pub fn workspace_api_handler(&self, body: &Bytes) -> Result<AppResponse> {
        let request: WorkspaceRequest =
            serde_json::from_slice(body).context("failed to parse request")?;

        let response = match request {
            WorkspaceRequest::ListDocuments { query } => {
                let mut filter = Filter::default();
                filter = filter.search(query.unwrap_or_default());

                let page = self.arhiv.list_documents(filter)?;

                WorkspaceResponse::ListDocuments {
                    documents: page
                        .items
                        .iter()
                        .map(|item| serde_json::to_string_pretty(item).unwrap())
                        .collect(),
                }
            }
            WorkspaceRequest::GetStatus {} => {
                let status = self.arhiv.get_status()?;

                WorkspaceResponse::GetStatus {
                    status: status.to_string(),
                }
            }
        };

        let response = serde_json::to_string(&response).context("failed to serialize response")?;

        Ok(AppResponse::json(response))
    }
}
