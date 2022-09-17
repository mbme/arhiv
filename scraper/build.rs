use std::env;

use rs_utils::run_package_json_script;

fn main() {
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-changed=src");

    // build app in release mode
    if env::var("PROFILE").unwrap() == "release" {
        run_package_json_script("install");
        run_package_json_script("prod:build");
        return;
    }

    run_package_json_script("build");
}
