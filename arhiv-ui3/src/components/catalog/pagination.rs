use maud::{html, Markup};

use rs_utils::server::Url;

pub fn render_pagination(url: &Url, page: u8, has_more: bool) -> Markup {
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

    html! {
        div
            class="my-6 flex justify-center items-center" {
                @if prev_page_url.is_some() {
                    a
                        href=(prev_page_url.unwrap())
                        class="mr-4 uppercase"
                        data-js="arhiv_ui.initCatalogLoadMore(this)" {
                            svg class="h-5 w-5 inline-block align-text-top mr-1" {
                                use xlink:href="#icon-narrow-left";
                            }

                            "prev"
                        }
                }

                @if next_page_url.is_some() || page > 0 {
                    div class="uppercase mono" {
                        "page " (page)
                    }
                }

                @if next_page_url.is_some() {
                    a
                        href=(next_page_url.unwrap())
                        class="ml-4 uppercase"
                        data-js="arhiv_ui.initCatalogLoadMore(this)" {
                            "next"

                            svg class="h-5 w-5 inline-block align-text-top ml-1" {
                                use xlink:href="#icon-narrow-right";
                            }
                        }

                }
            }
    }
}
