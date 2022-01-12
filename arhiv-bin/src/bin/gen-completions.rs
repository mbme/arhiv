use std::env;

use clap_complete::{generate_to, Shell};

use arhiv_bin::build_app;

fn main() {
    let manifest_dir = env!(
        "CARGO_MANIFEST_DIR",
        "CARGO_MANIFEST_DIR env variable is missing"
    );
    let outdir = format!("{}/completions", manifest_dir);

    let mut app = build_app();

    let bin_name = app.get_bin_name().unwrap().to_string();

    generate_to(Shell::Bash, &mut app, &bin_name, &outdir)
        .expect("failed to generate Bash completions");

    generate_to(Shell::Zsh, &mut app, &bin_name, &outdir)
        .expect("failed to generate Zsh completions");
}
