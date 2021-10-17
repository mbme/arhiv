use anyhow::*;
use serde::Serialize;

use arhiv_core::{definitions::TASK_STATUS, entities::Document, Arhiv, Filter};
use serde_json::json;

use crate::{
    components::{CatalogEntries, DocumentDataViewer},
    template_fn,
};

template_fn!(render_template, "./task_group.html.tera");

const OPEN_GROUPS: &[&str] = &["Inbox", "InProgress", "Paused"];

pub fn render_project_view(document: &Document, arhiv: &Arhiv, pattern: &str) -> Result<String> {
    let mut content = DocumentDataViewer::new(document).render(arhiv)?;

    let filter = Filter::default()
        .with_collection_ref(&document.id)
        .search(pattern)
        .all_items();

    let result = arhiv.list_documents(&filter)?;

    let mut entries = CatalogEntries::new();
    entries.parent_collection = Some(document.id.clone());

    let groups = TASK_STATUS
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

    let groups = render_template(json!({ "groups": groups }))?;
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
