use arhiv_core::entities::Id;
use rs_utils::server::Url;

pub fn index_url() -> String {
    "/".to_string()
}

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

pub fn erase_document_url(id: &Id, parent_collection: &Option<Id>) -> String {
    document_url(id, parent_collection) + "/erase"
}

pub fn pick_file_modal_fragment_url(dir: impl Into<String>, show_hidden: bool) -> String {
    let mut url = Url::new("/modals/pick-file");
    url.set_query_param("dir", Some(dir.into()));
    url.set_query_param("show-hidden", show_hidden.then(|| "".to_string()));

    url.render()
}

pub fn pick_file_confirmation_modal_fragment_url(file: impl Into<String>) -> String {
    let mut url = Url::new("/modals/pick-file-confirmation");
    url.set_query_param("file", Some(file.into()));

    url.render()
}

pub fn pick_file_confirmation_handler_url() -> String {
    "/modals/pick-file-confirmation".to_string()
}
