use app_shell::AppShellBuilder;

fn main() {
    env_logger::init();

    let path_str = format!("{}/static/index.html", env!("CARGO_MANIFEST_DIR"));

    AppShellBuilder::create("v.poshta".to_string())
        .with_title("Poshta".to_string())
        .show_inspector()
        .load(path_str);
}
