use std::{cmp::Ordering, env, fs, ops::Not, path::Path};

use anyhow::*;
use hyper::{Body, Request, StatusCode};
use serde::Serialize;
use serde_json::json;

use rs_utils::{
    ensure_dir_exists, get_home_dir,
    server::{RequestQueryExt, ServerResponse},
};

use crate::{
    template_fn,
    urls::{pick_file_confirmation_modal_fragment_url, pick_file_modal_fragment_url},
    utils::render_content,
};

const DEFAULT_DIR: &str = "/";

template_fn!(render_template, "./pick_file_modal.html.tera");

pub async fn pick_file_modal(req: Request<Body>) -> ServerResponse {
    let mut url = req.get_url();

    let show_hidden = url.get_query_param("show-hidden").is_some();
    let dir: String = url.get_query_param("dir").map_or_else(
        || get_home_dir().unwrap_or_else(|| DEFAULT_DIR.to_string()),
        ToString::to_string,
    );

    ensure_dir_exists(&dir)?;

    let dir = fs::canonicalize(dir)?;

    let mut entries = list_entries(&dir, show_hidden)?;
    sort_entries(&mut entries);

    let toggle_hidden_url = {
        url.set_query_param("show-hidden", show_hidden.not().then(|| "".to_string()));

        url.render()
    };

    let content = render_template(json!({
        "show_hidden": show_hidden,
        "toggle_hidden_url": toggle_hidden_url,
        "dir": dir,
        "entries": entries,
    }))?;

    render_content(StatusCode::OK, content)
}

fn list_entries(dir: &Path, show_hidden: bool) -> Result<Vec<Entry>> {
    let mut result = vec![];

    if let Some(parent) = dir.parent() {
        let path = parent
            .to_str()
            .context("Failed to convert file path to string")?
            .to_string();

        result.push(Entry {
            is_dir: true,
            name: "..".to_string(),
            url: pick_file_modal_fragment_url(path, show_hidden),
            size: None,
        });
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;

        let name = entry
            .file_name()
            .to_str()
            .context("Failed to convert file name to string")?
            .to_string();

        // skip hidden files
        if !show_hidden && name.starts_with('.') {
            continue;
        }

        let path = entry
            .path()
            .to_str()
            .context("Failed to convert file path to string")?
            .to_string();

        let metadata = entry.metadata()?;

        let url = if metadata.is_dir() {
            pick_file_modal_fragment_url(path, show_hidden)
        } else {
            pick_file_confirmation_modal_fragment_url(path)
        };

        result.push(Entry {
            is_dir: metadata.is_dir(),
            name,
            url,
            size: metadata.is_file().then(|| metadata.len()),
        });
    }

    Ok(result)
}

fn sort_entries(entries: &mut Vec<Entry>) {
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });
}

#[derive(Serialize)]
struct Entry {
    is_dir: bool,
    name: String,
    url: String,
    size: Option<u64>,
    // type: file, dir, symlink
    // can read
}
