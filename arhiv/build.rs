use std::env;

use rs_utils::run_npm;

fn main() {
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-changed=src/ui");
    println!("cargo:rerun-if-changed=src/dto.ts");

    if env::var("PROFILE").unwrap() == "release" {
        // build web app in release mode
        run_npm(["run", "prod:build"]);
    }
}
