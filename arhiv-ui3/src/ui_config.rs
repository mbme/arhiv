pub struct CatalogConfig {
    pub group_by: Option<CatalogGroupBy>,
    pub preview: Option<&'static str>,
    pub fields: Vec<&'static str>,
}

pub struct CatalogGroupBy {
    pub field: &'static str,
    pub open_groups: Vec<&'static str>,
    pub skip_empty_groups: bool,
}

impl CatalogConfig {
    pub fn get_config(document_type: impl AsRef<str>) -> Self {
        let document_type = document_type.as_ref();

        if document_type == "project/task" {
            return CatalogConfig {
                group_by: Some(CatalogGroupBy {
                    field: "status",
                    open_groups: vec!["Inbox", "InProgress", "Paused"],
                    skip_empty_groups: true,
                }),
                preview: None,
                fields: vec![],
            };
        }

        if document_type == "book" {
            return CatalogConfig {
                group_by: None,
                preview: None,
                fields: vec!["authors"],
            };
        }

        CatalogConfig {
            group_by: None,
            preview: None,
            fields: vec![],
        }
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
