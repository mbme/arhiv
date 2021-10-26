use std::collections::HashMap;

use anyhow::*;
use hyper::{header, Body, Request, Response, StatusCode};
use routerify::RequestInfo;
use serde::Serialize;

use crate::QueryBuilder;

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

    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(format!("Something went wrong: {:?}", err)))
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
    pub fn get_query_param(&self, name: impl AsRef<str>) -> Option<&str> {
        self.query_params
            .get(name.as_ref())
            .map(|value| value.as_ref())
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
    pub fn render(self) -> String {
        let query = QueryBuilder::from_params(self.query_params).build();

        if query.is_empty() {
            self.path
        } else {
            format!("{}?{}", self.path, query)
        }
    }
}

pub trait RequestQueryExt {
    fn get_url(&self) -> Url;

    fn get_query_params(&self) -> HashMap<String, String>;

    fn get_query_param(&self, name: impl AsRef<str>) -> Option<String>;

    fn get_url_with_updated_query(&self, key: impl Into<String>, value: Option<String>) -> String;
}

impl RequestQueryExt for Request<Body> {
    fn get_url(&self) -> Url {
        let path = self.uri().path().to_string();

        let query_params = form_urlencoded::parse(self.uri().query().unwrap_or("").as_bytes())
            .into_owned()
            .collect();

        Url { path, query_params }
    }

    fn get_query_params(&self) -> HashMap<String, String> {
        form_urlencoded::parse(self.uri().query().unwrap_or("").as_bytes())
            .into_owned()
            .collect()
    }

    fn get_query_param(&self, name: impl AsRef<str>) -> Option<String> {
        let name = name.as_ref();

        form_urlencoded::parse(self.uri().query().unwrap_or("").as_bytes()).find_map(
            |(key, value)| {
                if key == name {
                    return Some(value.into());
                }

                None
            },
        )
    }

    fn get_url_with_updated_query(&self, key: impl Into<String>, value: Option<String>) -> String {
        let uri = self.uri();
        let key = key.into();

        let mut params = self.get_query_params();

        if let Some(value) = value {
            params.insert(key, value);
        } else {
            params.remove(&key);
        }

        if params.is_empty() {
            return uri.path().to_string();
        }

        let query = QueryBuilder::from_params(params).build();

        format!("{}?{}", uri.path(), query)
    }
}

#[cfg(test)]
mod tests {
    use hyper::Uri;

    use super::*;

    fn new_request(uri: &'static str) -> Request<Body> {
        let uri = Uri::from_static(uri);

        let mut request = Request::new(Body::empty());

        request.uri_mut().clone_from(&uri);

        request
    }

    #[test]
    fn test_get_url_with_updated_query() {
        {
            let r = new_request("/test");

            assert_eq!(
                r.get_url_with_updated_query("test", Some("value".to_string())),
                "/test?test=value".to_string()
            );
        }

        {
            let r = new_request("/test?test=other");

            assert_eq!(
                r.get_url_with_updated_query("test", Some("value".to_string())),
                "/test?test=value".to_string()
            );
        }

        {
            let r = new_request("/test?test=other");

            assert_eq!(
                r.get_url_with_updated_query("test", None),
                "/test".to_string()
            );
        }
    }
}
