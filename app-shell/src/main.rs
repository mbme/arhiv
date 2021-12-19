use std::sync::Arc;

use anyhow::{anyhow, bail, ensure, Context, Result};
use app_shell::*;
use async_trait::async_trait;
use rs_utils::log::setup_logger;
use serde_json::{value, Value};

#[tokio::main]
async fn main() {
    setup_logger();

    let path_str = format!("{}/static/app.html", env!("CARGO_MANIFEST_DIR"));

    let handler = Arc::new(Handler {});

    AppShellBuilder::create("v.app-shell.playground")
        .with_title("App Shell Playground")
        .start(AppSource::HTMLFile(path_str), handler)
        .await;
}

struct Handler {}

#[async_trait]
impl ActionHandler for Handler {
    async fn run(&self, action: String, _value: Value) -> Result<Value> {
        if action == "get_value" {
            return value::to_value("some value").context("must be able to serialize");
        }

        unreachable!("unknown action: {}", action)
    }
}
