use std::env;

use rs_utils::npm_run;

fn main() {
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=public");

    if env::var("PROFILE").unwrap() != "release" {
        return;
    }

    // build web app in release mode
    npm_run("install");
    npm_run("prod:build");
}
