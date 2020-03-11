use app_shell::AppShell;
use std::path::Path;

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let current_dir = format!("{}/src", manifest_dir);
    let path_str = format!("{}/index.html", current_dir);
    let html_file = Path::new(&path_str);

    AppShell::show("arhiv.desktop.gtk", &html_file).unwrap();
}
