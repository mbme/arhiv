use anyhow::*;
use serde_json::json;

use arhiv_core::entities::Id;

use crate::template_fn;

template_fn!(render_template, "./search.html.tera");

pub struct CatalogSearch {
    pub query_param: Option<&'static str>,
}

impl CatalogSearch {
    pub fn render(
        &self,
        pattern: &str,
        document_type: Option<&str>,
        parent_collection: &Option<Id>,
    ) -> Result<String> {
        let placeholder = format!("Search {}s", document_type.unwrap_or("document"));

        render_template(json!({
            "pattern": pattern,
            "query_param": self.query_param,
            "placeholder": placeholder,
            "document_type": document_type,
            "parent_collection": parent_collection,
        }))
    }
}
