use app_shell::*;
use serde_json::{value, Value};

fn main() {
    env_logger::init();

    let path_str = format!("{}/static/app.html", env!("CARGO_MANIFEST_DIR"));

    let builder = AppShellBuilder::create("v.app-shell.playground")
        .with_title("App Shell Playground")
        .with_action("get_value", move |_, _params: Value| {
            value::to_value("some value").unwrap()
        })
        .with_action("pick_files", move |context: _, _params: Value| {
            let files = context.pick_files(true);

            value::to_value(files).unwrap()
        });

    builder.start(AppSource::HTMLFile(path_str));
}
