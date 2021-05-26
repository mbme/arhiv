use anyhow::*;
use askama::Template;
use rocket::State;

use arhiv::{entities::*, markup::MarkupRenderer, Arhiv, Filter, ListPage, Matcher, OrderBy};

use crate::utils::TemplateContext;

pub struct CatalogEntry {
    id: Id,
    preview: String,
}

#[derive(Template)]
#[template(path = "catalog_page.html")]
pub struct CatalogPage {
    context: TemplateContext,
    document_type: String,
    page: ListPage<CatalogEntry>,
}

#[get("/catalogs/<document_type>")]
pub fn render_catalog_page(
    document_type: String,
    arhiv: State<Arhiv>,
    context: State<TemplateContext>,
) -> Result<CatalogPage> {
    let mut filter = Filter::default();
    filter.matchers.push(Matcher::Type {
        document_type: document_type.clone(),
    });
    filter.page_size = Some(12);
    filter.order.push(OrderBy::UpdatedAt { asc: false });

    let renderer = MarkupRenderer::new(&arhiv, &context.markup_render_options);

    let page = arhiv.list_documents(filter)?.map(|document| CatalogEntry {
        preview: renderer
            .get_preview(&document)
            .unwrap_or("No preview".to_string()),
        id: document.id,
    });

    Ok(CatalogPage {
        context: context.clone(),
        document_type,
        page,
    })
}
