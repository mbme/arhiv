use std::default::Default;

pub struct Matcher {
    pub selector: String,
    pub pattern: String,
}

pub struct DocumentFilter {
    pub document_type: Option<String>,
    pub page_offset: Option<u8>,
    pub page_size: Option<u8>,
    pub matcher: Option<Matcher>,
    pub skip_archived: Option<bool>,
    pub only_staged: Option<bool>,
}

impl Default for DocumentFilter {
    fn default() -> Self {
        DocumentFilter {
            document_type: None,
            page_offset: Some(0),
            page_size: Some(20),
            matcher: None,
            skip_archived: Some(true),
            only_staged: None,
        }
    }
}

pub const DOCUMENT_FILTER_STAGED: DocumentFilter = DocumentFilter {
    document_type: None,
    page_offset: None,
    page_size: None,
    matcher: None,
    skip_archived: None,
    only_staged: Some(true),
};

pub struct AttachmentFilter {
    pub page_offset: Option<u8>,
    pub page_size: Option<u8>,
    pub pattern: Option<String>,
}

impl Default for AttachmentFilter {
    fn default() -> Self {
        AttachmentFilter {
            page_offset: Some(0),
            page_size: Some(20),
            pattern: None,
        }
    }
}
