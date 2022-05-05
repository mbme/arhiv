use anyhow::{Context, Result};
use arhiv_core::ApiRequest;
use hyper::body::Bytes;

use crate::app::{App, AppResponse};

impl App {
    pub fn api_handler(&self, body: &Bytes) -> Result<AppResponse> {
        let request: ApiRequest =
            serde_json::from_slice(body).context("failed to parse request")?;

        let response = self.arhiv.handle_api_request(request)?;

        let response = serde_json::to_string(&response).context("failed to serialize response")?;

        Ok(AppResponse::json(response))
    }
}
