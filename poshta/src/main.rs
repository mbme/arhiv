use app_shell::AppShellBuilder;
use serde_json::{value, Value};
use std::rc::Rc;

mod auth;

#[tokio::main]
async fn main() {
    env_logger::init();

    let token = auth::auth().await;

    let path_str = format!("{}/static/index.html", env!("CARGO_MANIFEST_DIR"));

    let action_handler = Rc::new(move |action: String, params: Value| {
        println!("Got action {} with params {:?}", action, params);

        if action == "get_token" {
            return value::to_value(token.clone()).unwrap();
        }

        Value::Null
    });

    AppShellBuilder::create("v.poshta".to_string())
        .with_title("Poshta".to_string())
        .with_rpc(action_handler)
        .show_inspector()
        .load(path_str);
}
