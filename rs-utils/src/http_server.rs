use std::collections::HashMap;

use anyhow::{Context, Result};
use hyper::{header, Body, Request, Response, StatusCode};
use routerify::RequestInfo;
use serde::Serialize;

pub type ServerResponse = Result<Response<Body>>;

pub fn respond_with_status(status: StatusCode) -> ServerResponse {
    Response::builder()
        .status(status)
        .body(Body::empty())
        .context("failed to build response")
}

pub fn respond_not_found() -> ServerResponse {
    respond_with_status(StatusCode::NOT_FOUND)
}

pub fn respond_see_other(uri: impl Into<String>) -> ServerResponse {
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, uri.into())
        .body(Body::empty())
        .context("failed to build response")
}

pub fn respond_moved_permanently(uri: impl Into<String>) -> ServerResponse {
    Response::builder()
        .status(StatusCode::MOVED_PERMANENTLY)
        .header(header::LOCATION, uri.into())
        .body(Body::empty())
        .context("failed to build response")
}

#[must_use]
pub fn parse_urlencoded(data: &[u8]) -> HashMap<String, String> {
    form_urlencoded::parse(data).into_owned().collect()
}

#[allow(clippy::unused_async)]
pub async fn not_found_handler(_req: Request<Body>) -> ServerResponse {
    respond_not_found()
}

#[allow(clippy::unused_async)]
pub async fn logger_middleware(res: Response<Body>, info: RequestInfo) -> Result<Response<Body>> {
    log::debug!(
        "{} {} -> {}",
        info.method(),
        info.uri().path(),
        res.status()
    );

    Ok(res)
}

#[allow(clippy::unused_async)]
pub async fn error_handler(err: routerify::RouteError, info: RequestInfo) -> Response<Body> {
    log::error!("{} {} -> {:?}", info.method(), info.uri().path(), err);

    // TODO stacktrace/backtrace, prettier UI
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(format!("Something went wrong:\n{:?}", err)))
        .unwrap()
}

pub fn json_response(body: impl Serialize) -> ServerResponse {
    Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_string(&body)?))
        .context("failed to build response")
}

#[derive(Clone)]
pub struct Url {
    path: String,
    pub query_params: HashMap<String, String>,
}

impl Url {
    pub fn new(path: impl Into<String>) -> Self {
        Url {
            path: path.into(),
            query_params: HashMap::default(),
        }
    }

    pub fn get_query_param(&self, name: impl AsRef<str>) -> Option<&str> {
        self.query_params.get(name.as_ref()).map(AsRef::as_ref)
    }

    pub fn set_query_param(&mut self, key: impl Into<String>, value: Option<String>) {
        let key = key.into();

        if let Some(value) = value {
            self.query_params.insert(key, value);
        } else {
            self.query_params.remove(&key);
        }
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn render(self) -> String {
        let query = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(self.query_params)
            .finish();

        if query.is_empty() {
            self.path
        } else {
            format!("{}?{}", self.path, query)
        }
    }
}

pub trait RequestQueryExt {
    fn get_url(&self) -> Url;
}

impl RequestQueryExt for Request<Body> {
    fn get_url(&self) -> Url {
        let path = self.uri().path().to_string();

        let query_params = form_urlencoded::parse(self.uri().query().unwrap_or("").as_bytes())
            .into_owned()
            .collect();

        Url { path, query_params }
    }
}
