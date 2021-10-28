use maud::{html, Render};

pub fn render_search_input(
    pattern: &str,
    document_type: Option<&str>,
    url: &str,
    update_query: bool,
) -> String {
    let placeholder = format!("Search {}s", document_type.unwrap_or("document"));

    let html = html! {
        input
            .catalog-search-input
            type="search"
            name="pattern"
            value=(pattern)
            placeholder=(placeholder)

            data-js={"arhiv_ui.initCatalogSearch(this, '" (url) "', " (update_query) ")"};
    };

    html.render().into_string()
}
