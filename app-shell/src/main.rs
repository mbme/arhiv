use app_shell::*;
use serde_json::{value, Value};

fn init_logger() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();
}

fn main() {
    init_logger();

    let path_str = format!("{}/static/index.html", env!("CARGO_MANIFEST_DIR"));

    AppShellBuilder::create("v.app-shell.playground")
        .with_title("App Shell Playground")
        .with_action("get_value", move |_params: Value| {
            value::to_value("some value").unwrap()
        })
        .with_action("pick_files", move |_params: Value| {
            let files = pick_files(true);

            value::to_value(files).unwrap()
        })
        .show_inspector()
        .load(path_str);
}
