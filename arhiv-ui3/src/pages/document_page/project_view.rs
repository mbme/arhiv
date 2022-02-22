use anyhow::Result;
use serde::Serialize;
use serde_json::json;

use arhiv_core::{
    definitions::{TASK_STATUS, TASK_TYPE},
    entities::Document,
    Arhiv, Filter,
};
use rs_utils::http_server::Url;

use crate::{
    components::{render_search_input, CatalogEntries, DocumentDataViewer},
    template_fn,
};

template_fn!(render_template, "./project_view.html.tera");

const OPEN_GROUPS: &[&str] = &["Inbox", "InProgress", "Paused"];

pub fn render_project_view(document: &Document, arhiv: &Arhiv, url: &Url) -> Result<String> {
    let mut content = DocumentDataViewer::new(document).render(arhiv)?;

    let pattern = url
        .get_query_param("pattern")
        .unwrap_or_default()
        .to_string();

    let filter = Filter::default()
        .with_document_type(TASK_TYPE)
        .with_collection_ref(&document.id)
        .search(&pattern)
        .recently_updated_first()
        .all_items();

    let search_input = render_search_input(&pattern, Some(TASK_TYPE), url.path())?;

    let result = arhiv.list_documents(&filter)?;

    let mut entries = CatalogEntries::new();
    entries.parent_collection = Some(document.id.clone());

    let mut groups = TASK_STATUS
        .iter()
        .map(|task_status| -> Result<CatalogGroup> {
            let items = result
                .items
                .iter()
                .filter(|item| item.data.get_str("status") == Some(task_status))
                .collect::<Vec<_>>();

            let entries = entries.render(&items, arhiv)?;

            Ok(CatalogGroup {
                value: task_status,
                open: OPEN_GROUPS.contains(task_status),
                items: entries,
                items_count: items.len(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // always open first non-empty group
    if let Some(group) = groups.iter_mut().find(|group| group.items_count > 0) {
        group.open = true;
    }

    let groups = render_template(json!({
        "search_input": search_input,
        "groups": groups,
    }))?;
    content.push_str(&groups);

    Ok(content)
}

#[derive(Serialize)]
struct CatalogGroup {
    value: &'static str,
    open: bool,
    items: String,
    items_count: usize,
}
