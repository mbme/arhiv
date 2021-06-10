use std::collections::HashMap;

use anyhow::*;
use hyper::{Body, Response, StatusCode, Uri};

pub type AppResponse = Result<Response<Body>>;

pub fn not_found() -> AppResponse {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .context("failed to build response")
}

pub fn get_query_params(uri: &Uri) -> HashMap<String, String> {
    uri.query()
        .map(|v| form_urlencoded::parse(v.as_bytes()).into_owned().collect())
        .unwrap_or_else(HashMap::new)
}

pub fn update_query_param(uri: &Uri, key: impl Into<String>, value: Option<String>) -> String {
    let key = key.into();

    let mut params = get_query_params(uri);

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
