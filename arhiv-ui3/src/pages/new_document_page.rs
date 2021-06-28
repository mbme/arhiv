use std::collections::HashMap;

use anyhow::ensure;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::{json, Value};

use crate::{
    components::{Breadcrumbs, Editor},
    utils::render_page,
};
use arhiv_core::{
    entities::Document,
    schema::{DocumentData, SCHEMA},
};
use rs_utils::server::{RequestQueryExt, ServerResponse};

pub async fn new_document_page(req: Request<Body>) -> ServerResponse {
    let document_type = req
        .param("document_type")
        .expect("document_type must be present");

    ensure!(
        !SCHEMA
            .get_data_description_by_type(document_type)?
            .is_internal
    );

    let params = req.get_query_params();

    let data =
        SCHEMA.create_with_initial_values(document_type, params_into_document_data(params))?;

    let document = Document::new(document_type.clone(), data.into());

    let editor = Editor::new(&document)?.render()?;
    let breadcrumbs = Breadcrumbs::NewDocument(&document).render()?;

    render_page(
        "pages/new_document_page.html.tera",
        json!({
            "breadcrumbs": breadcrumbs,
            "editor": editor,
            "document_type": document_type,
        }),
    )
}

fn params_into_document_data(params: HashMap<String, String>) -> DocumentData {
    params
        .into_iter()
        .map(|(key, value)| (key, Value::String(value)))
        .collect()
}
