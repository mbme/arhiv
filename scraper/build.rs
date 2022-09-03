use std::env;

use rs_utils::npm_run;

fn main() {
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=dist");

    // build app in release mode
    if env::var("PROFILE").unwrap() == "release" {
        npm_run("install");
        npm_run("prod:build");
        return;
    }

    npm_run("build");
}
