use anyhow::Result;
use serde_json::{json, Value};

use arhiv_scraper::Scraper;

use crate::{
    app::{App, AppResponse},
    components::Ref,
    template_fn,
    utils::Fields,
};

template_fn!(render_modal, "./scrape_modal.html.tera");
template_fn!(render_modal_result, "./scrape_modal_result.html.tera");

impl App {
    pub fn scrape_modal() -> Result<AppResponse> {
        let content = render_modal(json!({
            "url": "",
            "error": Value::Null,
        }))?;

        Ok(AppResponse::fragment(content))
    }

    pub async fn scrape_modal_handler(&self, fields: &Fields) -> Result<AppResponse> {
        let url = fields.get("url").map(|url| url.trim()).unwrap_or_default();

        let scraper = Scraper::new(&self.arhiv)?;
        let content = match scraper.scrape(url).await {
            Ok(documents) => {
                let refs = documents
                    .into_iter()
                    .map(|document| Ref::from_document(document).render(&self.arhiv))
                    .collect::<Result<Vec<_>>>()?;

                render_modal_result(json!({
                    "url": url,
                    "refs": refs,
                }))?
            }
            Err(error) => render_modal(json!({
                "url": url,
                "error": format!("{:?}", error),
            }))?,
        };

        Ok(AppResponse::fragment(content))
    }
}
