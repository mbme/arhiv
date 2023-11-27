use std::env;

use rs_utils::run_npm;

fn main() {
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-changed=src");

    // build app in release mode
    if env::var("PROFILE").unwrap() == "release" {
        run_npm(["ci"]);
        run_npm(["run", "prod:build"]);
        return;
    }

    run_npm(["run", "build"]);
}
