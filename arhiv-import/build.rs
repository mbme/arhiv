use std::env;

use rs_utils::{create_dir_if_not_exist, create_file_if_not_exist, current_dir_relpath, run_yarn};

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
    create_dir_if_not_exist(current_dir_relpath("dist"))
        .expect("failed to create dir if not exist");
    create_file_if_not_exist(current_dir_relpath("dist/bundle.js"))
        .expect("failed to create file if not exist");
}
