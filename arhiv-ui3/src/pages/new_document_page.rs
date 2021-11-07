use anyhow::*;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::json;

use arhiv_core::{
    entities::{Document, Id},
    schema::Collection,
    Arhiv, FieldValidationErrors,
};
use rs_utils::server::ServerResponse;

use crate::{
    components::{Breadcrumb, DocumentDataEditor, Toolbar},
    pages::base::render_page,
    template_fn,
    urls::{document_url, parent_collection_url},
};

template_fn!(render_template, "./new_document_page.html.tera");

pub async fn new_document_page(req: Request<Body>) -> ServerResponse {
    let document_type = req
        .param("document_type")
        .expect("document_type must be present");

    let parent_collection: Option<Id> = req
        .param("collection_id")
        .map(|collection_id| collection_id.into());

    let arhiv: &Arhiv = req.data().unwrap();

    let schema = arhiv.get_schema();

    ensure!(!schema.is_internal_type(document_type));

    let mut document = Document::new(document_type.clone());

    if let Some(ref parent_collection) = parent_collection {
        let collection = arhiv.must_get_document(parent_collection)?;
        let data_description = schema.get_data_description(&collection.document_type)?;

        if let Collection::Type {
            document_type: item_type,
            field,
        } = data_description.collection_of
        {
            ensure!(
                item_type == document_type,
                "collection_id is not a collection of {}",
                document_type
            );
            document.data.set(field, parent_collection);
        } else {
            bail!("collection_id is not a collection");
        };
    }

    let content = render_new_document_page_content(&document, &None, &parent_collection, arhiv)?;

    render_page(content, arhiv)
}

pub fn render_new_document_page_content(
    document: &Document,
    errors: &Option<FieldValidationErrors>,
    parent_collection: &Option<Id>,
    arhiv: &Arhiv,
) -> Result<String> {
    let editor = DocumentDataEditor::new(
        &document.data,
        arhiv
            .get_schema()
            .get_data_description(&document.document_type)?,
    )?
    .with_errors(errors)
    .render(document_url(&document.id, parent_collection))?;

    let toolbar = Toolbar::new()
        .with_breadcrumb(Breadcrumb::for_collection(
            &document.document_type,
            parent_collection,
            arhiv,
        )?)
        .with_breadcrumb(Breadcrumb::string(format!(
            "new {}",
            document.document_type
        )))
        .on_close(parent_collection_url(
            &document.document_type,
            parent_collection,
        ))
        .render()?;

    render_template(json!({
        "toolbar": toolbar,
        "editor": editor,
    }))
}
