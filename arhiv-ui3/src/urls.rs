use arhiv_core::entities::Id;

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

pub fn new_document_url(document_type: &str, parent_collection: &Option<Id>) -> String {
    if let Some(collection_id) = parent_collection {
        format!("/collections/{}/new/{}", collection_id, document_type)
    } else {
        format!("/new/{}", document_type)
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

pub fn delete_document_url(id: &Id, parent_collection: &Option<Id>) -> String {
    document_url(id, parent_collection) + "/delete"
}
