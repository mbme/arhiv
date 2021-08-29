pub struct CatalogConfig {
    pub group_by: Option<CatalogGroupBy>,
    pub preview: Option<&'static str>,
    pub fields: Vec<&'static str>,
}

impl Default for CatalogConfig {
    fn default() -> Self {
        CatalogConfig {
            group_by: None,
            preview: None,
            fields: vec![],
        }
    }
}

pub struct CatalogGroupBy {
    pub field: &'static str,
    pub open_groups: Vec<&'static str>,
    pub skip_empty_groups: bool,
}
