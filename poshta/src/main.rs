use app_shell::AppShellBuilder;
use serde_json::{value, Value};

mod auth;

#[tokio::main]
async fn main() {
    env_logger::init();

    let token = auth::auth().await;

    let path_str = format!("{}/static/index.html", env!("CARGO_MANIFEST_DIR"));

    AppShellBuilder::create("v.poshta")
        .with_title("Poshta")
        .with_action("get_token", move |_params: Value| {
            value::to_value(token.clone()).unwrap()
        })
        .show_inspector()
        .load(path_str);
}
