use std::collections::HashMap;

use form_urlencoded::Serializer;

pub mod server;

pub fn get_mime_from_path(path: impl AsRef<str>) -> String {
    mime_guess::from_path(path.as_ref())
        .first_or_octet_stream()
        .to_string()
}

pub struct QueryBuilder<'s> {
    serializer: Serializer<'s, String>,
}

impl<'s> QueryBuilder<'s> {
    pub fn new() -> Self {
        QueryBuilder {
            serializer: Serializer::new(String::new()),
        }
    }

    pub fn from_params(params: HashMap<String, String>) -> Self {
        let mut serializer = Serializer::new(String::new());
        serializer.extend_pairs(params);

        QueryBuilder { serializer }
    }

    pub fn add_param(&mut self, param: impl AsRef<str>, value: impl AsRef<str>) -> &mut Self {
        self.serializer.append_pair(param.as_ref(), value.as_ref());

        self
    }

    pub fn maybe_add_param(
        &mut self,
        param: impl AsRef<str>,
        value: Option<impl AsRef<str>>,
    ) -> &mut Self {
        if let Some(value) = value {
            self.serializer.append_pair(param.as_ref(), value.as_ref());
        }

        self
    }

    pub fn build(&mut self) -> String {
        self.serializer.finish()
    }
}
