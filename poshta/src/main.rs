use app_shell::{AppShellBuilder, AppSource};
use serde_json::{value, Value};

mod auth;

#[tokio::main]
async fn main() {
    env_logger::init();

    let token = auth::auth().await;

    AppShellBuilder::create("v.poshta")
        .with_title("Poshta")
        .with_action("get_token", move |_, _params: Value| {
            value::to_value(token.clone()).unwrap()
        })
        .start(AppSource::JSFile(format!(
            "{}/dist/bundle.js",
            env!("CARGO_MANIFEST_DIR")
        )));
}
