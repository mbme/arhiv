use anyhow::Result;
use hyper::StatusCode;

use arhiv_core::{
    entities::{Document, Id},
    Validator,
};

use crate::{
    app::{App, AppResponse},
    urls::document_url,
    utils::{fields_to_document_data, Fields},
};

impl App {
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

            return Ok(AppResponse::Content {
                content,
                status: StatusCode::UNPROCESSABLE_ENTITY,
            });
        }

        self.arhiv.tx_stage_document(&mut document, &mut tx)?;

        tx.commit()?;

        let location = document_url(&document.id, parent_collection);

        Ok(AppResponse::SeeOther { location })
    }
}
