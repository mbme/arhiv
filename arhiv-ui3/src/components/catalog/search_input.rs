use maud::{html, Markup};

pub fn render_search_input(
    pattern: &str,
    document_type: Option<&str>,
    url: &str,
    update_query: bool,
) -> Markup {
    let placeholder = format!("Search {}s", document_type.unwrap_or("document"));

    html! {
        input
            class="w-full mb-8"
            type="search"
            name="pattern"
            value=(pattern)
            placeholder=(placeholder)

            data-js={"arhiv_ui.initCatalogSearch(this, '" (url) "', " (update_query) ")"};
    }
}
