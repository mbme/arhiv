use std::collections::HashMap;

use anyhow::*;
use hyper::{Body, Request, Response, StatusCode, Uri};

pub type AppResponse = Result<Response<Body>>;

pub fn not_found() -> AppResponse {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .context("failed to build response")
}

pub trait RequestQueryExt {
    fn get_query_param(&self, name: impl AsRef<str>) -> Option<String>;
}

impl RequestQueryExt for Request<Body> {
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
}

fn extract_query_params(uri: &Uri) -> HashMap<String, String> {
    form_urlencoded::parse(uri.query().unwrap_or("").as_bytes())
        .into_owned()
        .collect()
}

pub fn update_query_param(uri: &Uri, key: impl Into<String>, value: Option<String>) -> String {
    let key = key.into();

    let mut params = extract_query_params(uri);

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

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use super::*;

    #[test]
    fn test_update_query_param() {
        {
            let uri: Uri = "/test".try_into().unwrap();

            assert_eq!(
                update_query_param(&uri, "test", Some("value".to_string())),
                "/test?test=value".to_string()
            );
        }

        {
            let uri: Uri = "/test?test=other".try_into().unwrap();

            assert_eq!(
                update_query_param(&uri, "test", Some("value".to_string())),
                "/test?test=value".to_string()
            );
        }

        {
            let uri: Uri = "/test?test=other".try_into().unwrap();

            assert_eq!(update_query_param(&uri, "test", None), "/test".to_string());
        }
    }
}
