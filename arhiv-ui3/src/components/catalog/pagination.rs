use anyhow::*;
use serde_json::json;

use rs_utils::server::Url;

use crate::template_fn;

template_fn!(render_template, "./pagination.html.tera");

pub fn render_pagination(url: &Url, page: u8, has_more: bool) -> Result<String> {
    let prev_page_url = (page > 0).then(|| {
        let mut url = url.clone();

        let prev_page = (page - 1).to_string();
        let prev_page =
            Some(prev_page).and_then(|value| if value == "0" { None } else { Some(value) });

        url.set_query_param("page", prev_page);

        url.render()
    });

    let next_page_url = has_more.then(|| {
        let mut url = url.clone();

        let next_page = (page + 1).to_string();

        url.set_query_param("page", Some(next_page));

        url.render()
    });

    render_template(json!({
        "prev_page_url": prev_page_url,
        "page": page,
        "next_page_url": next_page_url,
    }))
}
