use arhiv_core::entities::Id;
use rs_utils::QueryBuilder;

pub fn document_url(id: &Id, parent_collection: &Option<Id>) -> String {
    if let Some(collection_id) = parent_collection {
        format!("/collections/{}/documents/{}", collection_id, id)
    } else {
        format!("/documents/{}", id)
    }
}

pub fn document_editor_url(id: &Id, parent_collection: &Option<Id>) -> String {
    document_url(id, parent_collection) + "/edit"
}

pub fn document_archive_url(id: &Id, parent_collection: &Option<Id>) -> String {
    document_url(id, parent_collection) + "/archive"
}

pub fn document_delete_url(id: &Id, parent_collection: &Option<Id>) -> String {
    document_url(id, parent_collection) + "/delete"
}

pub enum NewDocumentUrl<'a> {
    Document(&'a str),
    CollectionItem(&'a str, &'a str, &'a Id),
}

impl<'a> NewDocumentUrl<'a> {
    pub fn build(&self) -> String {
        match self {
            NewDocumentUrl::Document(document_type) => {
                format!("/new/{}", document_type)
            }
            NewDocumentUrl::CollectionItem(document_type, field, id) => {
                let query = QueryBuilder::new()
                    .add_param(field, id)
                    .add_param("parent_collection", id)
                    .build();

                format!("/new/{}?{}", document_type, query)
            }
        }
    }
}

pub fn catalog_url(document_type: &str) -> String {
    format!("/catalogs/{}", document_type)
}

#[allow(clippy::option_if_let_else)]
pub fn parent_collection_url(document_type: &str, parent_collection: &Option<Id>) -> String {
    if let Some(ref collection_id) = parent_collection {
        document_url(collection_id, &None)
    } else {
        catalog_url(document_type)
    }
}

pub fn catalog_fragment_url(parent_collection: &Option<Id>) -> String {
    let mut url = "/fragments/catalog".to_string();

    if let Some(ref collection_id) = parent_collection {
        url.push_str("?parent_collection=");
        url.push_str(collection_id);
    }

    url
}
