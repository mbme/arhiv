use chrono::{DateTime, Local};
use hyper::{Body, Request};
use routerify::ext::RequestExt;
use serde::Serialize;
use serde_json::json;

use arhiv::{entities::*, Filter, Matcher, OrderBy};

use crate::{
    app_context::AppContext,
    components::Catalog,
    http_utils::{update_query_param, AppResponse, RequestQueryExt},
};

const PAGE_SIZE: u8 = 14;

#[derive(Serialize)]
struct CatalogEntry {
    id: Id,
    document_type: String,
    preview: String,
    updated_at: DateTime<Local>,
}

pub async fn catalog_page(req: Request<Body>) -> AppResponse {
    let document_type: &String = req.param("document_type").unwrap();
    let context: &AppContext = req.data().unwrap();

    let page: u8 = req
        .get_query_param("page")
        .unwrap_or("0".to_string())
        .parse()?;

    let pattern = req.get_query_param("pattern").unwrap_or("".to_string());

    let filter = catalog_filter(document_type, &pattern, page);

    let result = context.arhiv.list_documents(filter)?;
    let catalog = Catalog::new(result.items, pattern).render(&context)?;

    let prev_link = match page {
        0 => None,
        1 => Some(update_query_param(req.uri(), "page", None)),
        _ => Some(update_query_param(
            req.uri(),
            "page",
            Some((page - 1).to_string()),
        )),
    };

    let next_link = if result.has_more {
        Some(update_query_param(
            req.uri(),
            "page",
            Some((page + 1).to_string()),
        ))
    } else {
        None
    };

    context.render_page(
        "pages/catalog_page.html.tera",
        json!({
            "document_type": document_type,
            "catalog": catalog,
            "prev_link": prev_link,
            "page": page,
            "next_link": next_link,
        }),
    )
}

fn catalog_filter(
    document_type: impl Into<String>,
    pattern: impl Into<String>,
    page: u8,
) -> Filter {
    let mut filter = Filter::default();
    filter.matchers.push(Matcher::Type {
        document_type: document_type.into(),
    });
    filter.matchers.push(Matcher::Search {
        pattern: pattern.into(),
    });
    filter.page_size = Some(PAGE_SIZE);
    filter.page_offset = Some(PAGE_SIZE * page);
    filter.order.push(OrderBy::UpdatedAt { asc: false });

    filter
}
