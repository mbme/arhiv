use app_shell::AppShellBuilder;
use std::rc::Rc;

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let current_dir = format!("{}/src", manifest_dir);
    let path_str = format!("{}/index.html", current_dir);

    let action_handler = Rc::new(|action, params| {
        println!("Got action {} with params {}", action, params);

        "{}".to_string()
    });

    AppShellBuilder::create("arhiv.desktop.gtk".to_string())
        .with_title("arhiv desktop test".to_string())
        .with_rpc(action_handler)
        .show_inspector()
        .load(path_str);
}
