use std::default::Default;

pub enum QueryMode {
    All,
    Commited,
}

pub struct QueryFilter {
    pub document_type: Option<String>,
    pub page_offset: Option<u8>,
    pub page_size: Option<u8>,
}

impl Default for QueryFilter {
    fn default() -> Self {
        QueryFilter {
            document_type: None,
            page_offset: Some(0),
            page_size: Some(20),
        }
    }
}

pub struct QueryPage<T> {
    pub results: Vec<T>,
    pub total: u16,
    pub page_offset: Option<u8>,
    pub page_size: Option<u8>,
}
