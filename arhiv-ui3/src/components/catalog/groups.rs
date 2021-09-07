use std::collections::HashMap;

use anyhow::*;
use serde::Serialize;
use serde_json::json;

use arhiv_core::{
    entities::{Document, Id},
    Arhiv,
};

use crate::template_fn;

use super::{
    config::{CatalogConfig, CatalogGroupBy},
    entries::{render_entries, CatalogEntry},
};

template_fn!(render_template, "./groups.html.tera");

#[derive(Serialize)]
struct CatalogGroup {
    value: &'static str,
    open: bool,
    items: String,
    items_count: usize,
}

pub fn group_documents(
    documents: Vec<Document>,
    arhiv: &Arhiv,
    config: &CatalogConfig,
    field: &str,
    collection_id: &Option<Id>,
) -> Result<HashMap<String, Vec<CatalogEntry>>> {
    let mut result = HashMap::new();

    for document in documents {
        let key = document
            .data
            .get_str(field)
            .ok_or_else(|| anyhow!("can't find field"))?
            .to_string();

        let entry = result.entry(key).or_insert_with(Vec::new);

        entry.push(CatalogEntry::new(document, arhiv, config, collection_id)?);
    }

    Ok(result)
}

pub fn render_groups(
    group_names: &[&'static str],
    mut documents: HashMap<String, Vec<CatalogEntry>>,
    group_by: &CatalogGroupBy,
) -> Result<String> {
    let mut groups = group_names
        .iter()
        .map(|group_name| {
            let entries = documents.remove(*group_name).unwrap_or_default();
            let items_count = entries.len();
            let items = render_entries(&entries)?;

            Ok(CatalogGroup {
                value: group_name,
                items,
                items_count,
                open: group_by.open_groups.contains(group_name),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // skip empty groups if needed
    if group_by.skip_empty_groups {
        groups.retain(|group| !group.items.is_empty());
    }

    // open first non-empty group if no groups open yet
    if !groups.iter().any(|group| group.open) {
        if let Some(group) = groups.iter_mut().find(|group| !group.items.is_empty()) {
            group.open = true;
        }
    }

    render_template(json!({ "groups": groups }))
}
