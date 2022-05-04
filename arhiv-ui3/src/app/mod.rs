use std::sync::Arc;

use hyper::StatusCode;
use serde_json::json;

use arhiv_core::{
    definitions::{ATTACHMENT_TYPE, TASK_TYPE},
    Arhiv,
};
use rs_utils::{
    capitalize,
    http_server::{
        respond_moved_permanently, respond_see_other, respond_with_status, ServerResponse,
    },
};

use crate::{
    template_fn,
    urls::catalog_url,
    utils::{render_content, render_json},
};

template_fn!(render_template, "./base.html.tera");
template_fn!(render_icons, "./icons.html.tera");

pub struct App {
    pub arhiv: Arc<Arhiv>,
    nav_document_types: Vec<(&'static str, String)>,
}

impl App {
    pub fn new(arhiv: Arhiv) -> App {
        App {
            nav_document_types: get_nav_document_types(&arhiv),
            arhiv: Arc::new(arhiv),
        }
    }

    fn render_page_with_status(
        &self,
        status: StatusCode,
        title: &str,
        content: &str,
    ) -> ServerResponse {
        let icons = render_icons(json!({})).expect("failed to render icons");

        let result = render_template(json!({
            "title": capitalize(title),
            "nav_document_types": self.nav_document_types,
            "content": content,
            "icons": icons,
        }))?;

        render_content(status, result)
    }

    pub fn render(&self, response: AppResponse) -> ServerResponse {
        match response {
            AppResponse::Page {
                title,
                content,
                status,
            } => self.render_page_with_status(status, &title, &content),
            AppResponse::Json { content } => render_json(StatusCode::OK, content),
            AppResponse::Fragment { content } => render_content(StatusCode::OK, content),
            AppResponse::Status { status } => respond_with_status(status),
            AppResponse::SeeOther { location } => respond_see_other(location),
            AppResponse::MovedPermanently { location } => respond_moved_permanently(location),
        }
    }
}

pub enum AppResponse {
    Page {
        title: String,
        content: String,
        status: StatusCode,
    },
    Fragment {
        content: String,
    },
    Status {
        status: StatusCode,
    },
    Json {
        content: String,
    },
    SeeOther {
        location: String,
    },
    MovedPermanently {
        location: String,
    },
}

impl AppResponse {
    pub fn page(title: String, content: String) -> Self {
        AppResponse::Page {
            title,
            content,
            status: StatusCode::OK,
        }
    }

    pub fn page_with_status(title: String, content: String, status: StatusCode) -> Self {
        AppResponse::Page {
            title,
            content,
            status,
        }
    }

    pub fn fragment(content: String) -> Self {
        AppResponse::Fragment { content }
    }

    pub fn status(status: StatusCode) -> Self {
        AppResponse::Status { status }
    }

    pub fn json(content: String) -> Self {
        AppResponse::Json { content }
    }
}

const IGNORED_DOCUMENT_TYPES: &[&str] = &[ATTACHMENT_TYPE, TASK_TYPE];

fn get_nav_document_types(arhiv: &Arhiv) -> Vec<(&'static str, String)> {
    arhiv
        .get_schema()
        .get_document_types()
        .into_iter()
        .filter(|document_type| !IGNORED_DOCUMENT_TYPES.contains(document_type))
        .map(|module| (module, catalog_url(module)))
        .collect()
}
