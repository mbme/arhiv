use anyhow::{bail, ensure, Result};
use hyper::StatusCode;
use serde_json::json;

use arhiv_core::{
    entities::{Document, Id, ERASED_DOCUMENT_TYPE},
    schema::Collection,
    FieldValidationErrors, Validator,
};

use crate::{
    app::{App, AppResponse},
    components::{Breadcrumb, DocumentDataEditor, Toolbar},
    template_fn,
    urls::{document_url, parent_collection_url},
    utils::{fields_to_document_data, Fields},
};

template_fn!(render_template, "./new_document_page.html.tera");

impl App {
    pub fn new_document_page(
        &self,
        document_type: &str,
        parent_collection: &Option<Id>,
    ) -> Result<AppResponse> {
        let schema = self.arhiv.get_schema();

        ensure!(document_type != ERASED_DOCUMENT_TYPE);

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

        let title = format!("New {}", document_type);

        Ok(AppResponse::page(title, content))
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

    pub fn new_document_page_handler(
        &self,
        document_type: &str,
        parent_collection: &Option<Id>,
        fields: &Fields,
    ) -> Result<AppResponse> {
        let data_description = self
            .arhiv
            .get_schema()
            .get_data_description(document_type)?;

        let data = fields_to_document_data(fields, data_description)?;

        let mut document = Document::new_with_data(document_type, data);

        let mut tx = self.arhiv.get_tx()?;
        let validation_result =
            Validator::default().validate(&document.data, None, data_description, &mut tx);

        if let Err(error) = validation_result {
            tx.commit()?;

            let content = self.render_new_document_page_content(
                &document,
                parent_collection,
                &Some(error.errors),
            )?;

            let title = format!("New {}", document_type);

            return Ok(AppResponse::page_with_status(
                title,
                content,
                StatusCode::UNPROCESSABLE_ENTITY,
            ));
        }

        self.arhiv.tx_stage_document(&mut document, &mut tx)?;

        tx.commit()?;

        let location = document_url(&document.id, parent_collection);

        Ok(AppResponse::SeeOther { location })
    }
}
