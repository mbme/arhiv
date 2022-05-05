use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{Arhiv, Filter, ListPage};

#[derive(Deserialize, Debug)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum ApiRequest {
    ListDocuments { filter: Filter },
}

#[derive(Serialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum ApiResponse {
    ListDocuments { page: ListPage },
}

impl Arhiv {
    pub fn handle_api_request(&self, request: ApiRequest) -> Result<ApiResponse> {
        let response = match request {
            ApiRequest::ListDocuments { filter } => {
                let page = self.list_documents(filter)?;

                ApiResponse::ListDocuments { page }
            }
        };

        Ok(response)
    }
}
