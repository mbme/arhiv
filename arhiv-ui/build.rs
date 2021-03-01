use std::{env, process::Command};

// build web app in release mode
fn main() {
    if env::var("PROFILE").unwrap() != "release" {
        return;
    }

    let result = Command::new("yarn")
        .arg("build:release")
        .status()
        .expect("failed to build web app");

    println!("cargo:warning=result is {}", result);

    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-changed=src");
}
