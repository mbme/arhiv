use std::env;

use rs_utils::run_yarn;

fn main() {
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-changed=src");

    // build app in release mode
    if env::var("PROFILE").unwrap() == "release" {
        run_yarn("install");
        run_yarn("prod:build");
        return;
    }

    run_yarn("build");
}
