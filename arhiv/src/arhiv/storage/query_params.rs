use std::default::Default;

pub enum QueryMode {
    All,
    Commited,
}

pub struct QueryPage {
    pub offset: u8,
    pub size: u8,
}

impl Default for QueryPage {
    fn default() -> Self {
        QueryPage {
            offset: 0,
            size: 20,
        }
    }
}

pub struct QueryFilter {
    pub document_type: Option<String>,
    pub page: Option<QueryPage>,
}

impl Default for QueryFilter {
    fn default() -> Self {
        QueryFilter {
            document_type: None,
            page: Some(QueryPage::default()),
        }
    }
}
