use askama::Template;

use rocket::State;

use crate::utils::TemplateContext;

#[derive(Template)]
#[template(path = "not_found_page.html")]
pub struct NotFoundPage {
    context: TemplateContext,
}

#[catch(404)]
pub fn render_not_found_page(request: &rocket::Request) -> NotFoundPage {
    let context: State<TemplateContext> = request.guard().unwrap();

    NotFoundPage {
        context: context.clone(),
    }
}
