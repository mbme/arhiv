use std::collections::HashMap;

use anyhow::*;
use hyper::{Body, Request, Response, StatusCode};

pub type AppResponse = Result<Response<Body>>;

pub fn not_found() -> AppResponse {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .context("failed to build response")
}

pub trait RequestQueryExt {
    fn get_query_params(&self) -> HashMap<String, String>;

    fn get_query_param(&self, name: impl AsRef<str>) -> Option<String>;

    fn get_url_with_updated_query(&self, key: impl Into<String>, value: Option<String>) -> String;
}

impl RequestQueryExt for Request<Body> {
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

        let query = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();

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
