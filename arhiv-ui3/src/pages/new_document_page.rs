use anyhow::{bail, ensure, Result};
use serde_json::json;

use arhiv_core::{
    entities::{Document, Id},
    schema::Collection,
    FieldValidationErrors,
};

use crate::{
    app::{App, AppResponse},
    components::{Breadcrumb, DocumentDataEditor, Toolbar},
    template_fn,
    urls::parent_collection_url,
};

template_fn!(render_template, "./new_document_page.html.tera");

impl App {
    pub fn new_document_page(
        &self,
        document_type: &str,
        parent_collection: &Option<Id>,
    ) -> Result<AppResponse> {
        let schema = self.arhiv.get_schema();

        ensure!(!schema.is_internal_type(document_type));

        let mut document = Document::new(document_type);

        if let Some(ref parent_collection) = parent_collection {
            let collection = self.arhiv.must_get_document(parent_collection)?;
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

        let content = self.render_new_document_page_content(&document, parent_collection, &None)?;

        Ok(AppResponse::page(content))
    }

    pub fn render_new_document_page_content(
        &self,
        document: &Document,
        parent_collection: &Option<Id>,
        errors: &Option<FieldValidationErrors>,
    ) -> Result<String> {
        let cancel_url = parent_collection_url(&document.document_type, parent_collection);

        let editor = DocumentDataEditor::new(
            &document.data,
            self.arhiv
                .get_schema()
                .get_data_description(&document.document_type)?,
        )?
        .with_errors(errors)
        .render(cancel_url)?;

        let toolbar = Toolbar::new()
            .with_breadcrumb(Breadcrumb::for_collection(
                &document.document_type,
                parent_collection,
                &self.arhiv,
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
}
