use std::env;

use rs_utils::run_package_json_script;

fn main() {
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=public");

    if env::var("PROFILE").unwrap() != "release" {
        return;
    }

    // build web app in release mode
    run_package_json_script("install");
    run_package_json_script("prod:build");
}
