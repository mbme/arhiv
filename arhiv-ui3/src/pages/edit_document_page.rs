use anyhow::Result;
use hyper::StatusCode;
use serde_json::json;

use arhiv_core::{entities::*, FieldValidationErrors, Validator};

use crate::{
    app::{App, AppResponse},
    components::{Breadcrumb, DocumentDataEditor, Toolbar},
    template_fn,
    urls::document_url,
    utils::{fields_to_document_data, Fields},
};

template_fn!(render_template, "./edit_document_page.html.tera");

impl App {
    pub fn edit_document_page(
        &self,
        id: &Id,
        parent_collection: &Option<Id>,
    ) -> Result<AppResponse> {
        let document = {
            if let Some(document) = self.arhiv.get_document(id)? {
                document
            } else {
                return Ok(AppResponse::status(StatusCode::NOT_FOUND));
            }
        };

        let content =
            self.render_edit_document_page_content(&document, parent_collection, &None)?;

        Ok(AppResponse::content(content))
    }

    fn render_edit_document_page_content(
        &self,
        document: &Document,
        parent_collection: &Option<Id>,
        errors: &Option<FieldValidationErrors>,
    ) -> Result<String> {
        let editor = DocumentDataEditor::new(
            &document.data,
            self.arhiv
                .get_schema()
                .get_data_description(&document.document_type)?,
        )?
        .with_errors(errors)
        .render(document_url(&document.id, parent_collection))?;

        let toolbar = Toolbar::new()
            .with_breadcrumb(Breadcrumb::for_collection(
                &document.document_type,
                parent_collection,
                &self.arhiv,
            )?)
            .with_breadcrumb(Breadcrumb::for_document(document))
            .with_breadcrumb(Breadcrumb::string("editor"))
            .on_close(document_url(&document.id, parent_collection))
            .render()?;

        render_template(json!({
            "toolbar": toolbar,
            "editor": editor,
        }))
    }

    pub fn edit_document_page_handler(
        &self,
        id: &Id,
        parent_collection: &Option<Id>,
        fields: &Fields,
    ) -> Result<AppResponse> {
        let mut document = self.arhiv.must_get_document(id)?;

        let data_description = self
            .arhiv
            .get_schema()
            .get_data_description(&document.document_type)?;

        let prev_data = document.data;
        document.data = fields_to_document_data(fields, data_description)?;

        let mut tx = self.arhiv.get_tx()?;
        let validation_result = Validator::default().validate(
            &document.data,
            Some(&prev_data),
            data_description,
            &mut tx,
        );

        if let Err(error) = validation_result {
            tx.commit()?;

            let content = self.render_edit_document_page_content(
                &document,
                parent_collection,
                &Some(error.errors),
            )?;

            return Ok(AppResponse::Content {
                content,
                status: StatusCode::UNPROCESSABLE_ENTITY,
            });
        }

        self.arhiv.tx_stage_document(&mut document, &mut tx)?;

        tx.commit()?;

        let location = document_url(id, parent_collection);

        Ok(AppResponse::SeeOther { location })
    }
}
