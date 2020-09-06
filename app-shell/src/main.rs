use app_shell::*;
use serde_json::{value, Value};

fn main() {
    env_logger::init();

    let path_str = format!("{}/static/app.html", env!("CARGO_MANIFEST_DIR"));

    let builder = AppShellBuilder::create("v.app-shell.playground")
        .with_title("App Shell Playground")
        .with_action("get_value", move |_params: Value| {
            value::to_value("some value").unwrap()
        })
        .with_action("pick_files", move |_params: Value| {
            let files = pick_files(true);

            value::to_value(files).unwrap()
        });

    if option_env!("SERVER").is_some() {
        builder.serve(AppSource::HTMLFile(path_str));
    } else {
        builder.load(AppSource::HTMLFile(path_str));
    }
}
