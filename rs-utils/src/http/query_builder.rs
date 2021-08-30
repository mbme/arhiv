use std::collections::HashMap;

use form_urlencoded::Serializer;

pub struct QueryBuilder<'s> {
    serializer: Serializer<'s, String>,
}

impl<'s> QueryBuilder<'s> {
    #[must_use]
    pub fn new() -> Self {
        QueryBuilder {
            serializer: Serializer::new(String::new()),
        }
    }

    #[must_use]
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

impl<'s> Default for QueryBuilder<'s> {
    fn default() -> Self {
        Self::new()
    }
}
