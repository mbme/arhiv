use std::env;

use rs_utils::run_npm;

fn main() {
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-changed=src/ui");
    println!("cargo:rerun-if-changed=src/dto.ts");
    println!("cargo:rerun-if-changed=public");

    if env::var("PROFILE").unwrap() == "release" {
        // build web app in release mode
        run_npm(["run", "prod:build"]);
    } else if cfg!(feature = "debug-embed") {
        run_npm(["run", "build"]);
    }
}
