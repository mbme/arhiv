use std::collections::HashMap;

use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde_json::{json, Value};

use crate::{
    app_context::AppContext,
    components::Editor,
    http_utils::{AppResponse, RequestQueryExt},
};
use arhiv::{
    entities::Document,
    schema::{DocumentData, SCHEMA},
};

pub async fn new_document_page(req: Request<Body>) -> AppResponse {
    let document_type = req
        .param("document_type")
        .expect("document_type must be present");

    let context: &AppContext = req.data().unwrap();

    let params = req.get_query_params();

    let data =
        SCHEMA.create_with_initial_values(document_type, params_into_document_data(params))?;

    let document = Document::new(document_type.clone(), data.into());

    let editor = Editor::new(&document)?.render(&context)?;

    context.render_page(
        "pages/new_document_page.html.tera",
        json!({
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
