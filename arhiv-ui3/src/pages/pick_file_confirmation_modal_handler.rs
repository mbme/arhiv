use anyhow::*;
use hyper::{http::request::Parts, Body, Request, StatusCode};
use routerify::ext::RequestExt;
use serde_json::json;

use arhiv_core::Arhiv;
use rs_utils::server::ServerResponse;

use crate::{
    components::Ref,
    template_fn,
    utils::{extract_fields, render_content},
};

template_fn!(
    render_template,
    "./pick_file_confirmation_modal_handler.html.tera"
);

pub async fn pick_file_confirmation_modal_handler(req: Request<Body>) -> ServerResponse {
    let (parts, body): (Parts, Body) = req.into_parts();

    let arhiv: &Arhiv = parts.data().unwrap();

    let fields = extract_fields(body).await?;
    let file_path = fields
        .get("file_path")
        .ok_or_else(|| anyhow!("file_path field must be present"))?;

    let attachment = arhiv.add_attachment(file_path, false)?;
    let id = attachment.id.to_string();

    let attachment_ref = Ref::from_document(attachment.into()).render(arhiv)?;

    let content = render_template(json!({
        "id": id,
        "file_path": file_path,
        "attachment_ref": attachment_ref,
    }))?;

    render_content(StatusCode::OK, content)
}
