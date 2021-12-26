use std::{env, path::Path};

use rs_utils::{create_file_if_not_exist, run_yarn};

fn main() {
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=dist");

    // build app in release mode
    if env::var("PROFILE").unwrap() == "release" {
        run_yarn("install");
        run_yarn("prod:build");
        return;
    }

    // in dev mode create file if missing so that CI doesn't fail
    let file_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("dist/bundle.js");

    create_file_if_not_exist(file_path).expect("failed to create file if not exist");
}
