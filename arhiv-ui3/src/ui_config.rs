pub struct CatalogConfig {
    pub group_by: Option<CatalogGroupBy>,
}

pub struct CatalogGroupBy {
    pub field: &'static str,
}

impl CatalogConfig {
    pub fn get_config(document_type: impl AsRef<str>) -> Self {
        if document_type.as_ref() == "project/task" {
            return CatalogConfig {
                group_by: Some(CatalogGroupBy { field: "status" }),
            };
        }

        CatalogConfig { group_by: None }
    }

    pub fn get_child_config(
        parent_document_type: impl AsRef<str>,
        child_document_type: impl AsRef<str>,
    ) -> Self {
        CatalogConfig::get_config(format!(
            "{}/{}",
            parent_document_type.as_ref(),
            child_document_type.as_ref()
        ))
    }
}
