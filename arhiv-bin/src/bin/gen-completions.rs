use std::env;

use clap::Shell;

use arhiv_bin::build_app;

fn main() {
    let manifest_dir = env!(
        "CARGO_MANIFEST_DIR",
        "CARGO_MANIFEST_DIR env variable is missing"
    );
    let outdir = format!("{}/completions", manifest_dir);

    let mut app = build_app();

    let bin_name = app.get_bin_name().unwrap().to_string();

    app.gen_completions(&bin_name, Shell::Bash, outdir.clone());
    app.gen_completions(&bin_name, Shell::Zsh, outdir);
}
