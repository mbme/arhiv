use anyhow::*;
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde::Serialize;
use serde_json::json;

use crate::{
    components::{Breadcrumb, Catalog, Ref, Toolbar},
    markup::MarkupStringExt,
    ui_config::UIConfig,
    utils::render_page,
};
use arhiv_core::{
    entities::Document,
    markup::MarkupStr,
    schema::{DataDescription, FieldType, SCHEMA},
    Arhiv, Matcher,
};
use rs_utils::{
    query_builder,
    server::{respond_not_found, RequestQueryExt, ServerResponse},
};

#[derive(Serialize)]
enum FieldKind {
    Title,
    Markup,
    Ref,
    String,
}

#[derive(Serialize)]
struct Field {
    name: &'static str,
    value: String,
    kind: FieldKind,
}

pub async fn document_page(req: Request<Body>) -> ServerResponse {
    let id: &str = req.param("id").unwrap();
    let arhiv: &Arhiv = req.data().unwrap();

    let document = {
        if let Some(document) = arhiv.get_document(&id.into())? {
            document
        } else {
            return respond_not_found();
        }
    };

    let pattern = req.get_query_param("pattern").unwrap_or("".to_string());

    let data_description = SCHEMA.get_data_description(&document.document_type)?;
    let fields = prepare_fields(&document, arhiv, data_description)?;

    let mut children_catalog = None;

    if let Some(ref collection) = data_description.collection_of {
        let catalog = Catalog::new(collection.item_type, pattern)
            .with_matcher(Matcher::Field {
                selector: format!("$.{}", &document.document_type),
                pattern: document.id.to_string(),
            })
            .with_new_document_query(
                query_builder()
                    .append_pair(&document.document_type, &document.id)
                    .finish(),
            )
            .render(
                arhiv,
                UIConfig::get_child_config(&document.document_type, &collection.item_type).catalog,
            )?;

        children_catalog = Some(catalog);
    };

    let mut toolbar = Toolbar::new()
        .with_breadcrubs(vec![
            Breadcrumb::for_document_collection(&document)?,
            Breadcrumb::for_document(&document, false),
        ])
        .on_close_document(&document);

    if !data_description.is_internal {
        toolbar = toolbar.with_action("Edit", format!("/documents/{}/edit", &document.id));
    }

    let toolbar = toolbar.render()?;

    let refs = document
        .refs
        .iter()
        .map(|id| Ref::new(id).render(arhiv))
        .collect::<Result<Vec<_>>>()?;

    let backrefs = arhiv
        .get_document_backrefs(&document.id)?
        .into_iter()
        .map(|id| Ref::new(id).render(arhiv))
        .collect::<Result<Vec<_>>>()?;

    render_page(
        "pages/document_page.html.tera",
        json!({
            "toolbar": toolbar,
            "fields": fields,
            "refs": refs,
            "backrefs": backrefs,
            "document": document,
            "is_internal_type": data_description.is_internal,
            "children_catalog": children_catalog,
        }),
    )
}

fn prepare_fields(
    document: &Document,
    arhiv: &Arhiv,
    data_description: &DataDescription,
) -> Result<Vec<Field>> {
    let title_field = SCHEMA
        .get_data_description(&document.document_type)?
        .pick_title_field()?;

    data_description
        .fields
        .iter()
        .map(|field| {
            let value = document.get_field_str(field.name);
            let is_title = field.name == title_field.name;

            match (&field.field_type, value) {
                (FieldType::MarkupString {}, _) => {
                    let markup: MarkupStr = value.unwrap_or("").into();

                    Ok(Field {
                        name: field.name,
                        value: markup.to_html(arhiv),
                        kind: FieldKind::Markup,
                    })
                }
                (FieldType::Ref(_), Some(value)) => Ok(Field {
                    name: field.name,
                    value: Ref::new(value).preview_attachments().render(arhiv)?,
                    kind: FieldKind::Ref,
                }),
                _ => Ok(Field {
                    name: field.name,
                    value: value.unwrap_or("").to_string(),
                    kind: if is_title {
                        FieldKind::Title
                    } else {
                        FieldKind::String
                    },
                }),
            }
        })
        .collect::<Result<Vec<_>>>()
}
