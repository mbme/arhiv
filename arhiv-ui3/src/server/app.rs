use std::sync::Arc;

use hyper::StatusCode;

use arhiv_core::Arhiv;
use rs_utils::http_server::{respond_with_status, ServerResponse};

use super::utils::{render_content, render_json};

pub struct App {
    pub arhiv: Arc<Arhiv>,
}

impl App {
    pub fn new(arhiv: Arhiv) -> App {
        App {
            arhiv: Arc::new(arhiv),
        }
    }

    pub fn render(&self, response: AppResponse) -> ServerResponse {
        match response {
            AppResponse::Json { content } => render_json(StatusCode::OK, content),
            AppResponse::Fragment { content } => render_content(StatusCode::OK, content),
            AppResponse::Status { status } => respond_with_status(status),
        }
    }
}

pub enum AppResponse {
    Fragment { content: String },
    Status { status: StatusCode },
    Json { content: String },
}

impl AppResponse {
    pub fn fragment(content: String) -> Self {
        AppResponse::Fragment { content }
    }

    pub fn json(content: String) -> Self {
        AppResponse::Json { content }
    }
}
