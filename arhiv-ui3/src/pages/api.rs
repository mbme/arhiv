use anyhow::{Context, Result};
use hyper::body::Bytes;
use serde::{Deserialize, Serialize};

use arhiv_core::{Filter, ListPage};

use crate::app::{App, AppResponse};

#[derive(Deserialize, Debug)]
#[serde(tag = "type", deny_unknown_fields)]
enum ApiRequest {
    ListDocuments { filter: Filter },
    Status {},
}

#[derive(Serialize)]
#[serde(tag = "type", deny_unknown_fields)]
enum ApiResponse {
    ListDocuments { page: ListPage },
    Status { status: String },
}

impl App {
    pub fn api_handler(&self, body: &Bytes) -> Result<AppResponse> {
        let request: ApiRequest =
            serde_json::from_slice(body).context("failed to parse request")?;

        let response = match request {
            ApiRequest::ListDocuments { filter } => {
                let page = self.arhiv.list_documents(filter)?;

                ApiResponse::ListDocuments { page }
            }
            ApiRequest::Status {} => {
                let status = self.arhiv.get_status()?;

                ApiResponse::Status {
                    status: status.to_string(),
                }
            }
        };

        let response = serde_json::to_string(&response).context("failed to serialize response")?;

        Ok(AppResponse::json(response))
    }
}
