use std::{env, process};

// build web app in release mode
fn main() {
    if env::var("PROFILE").unwrap() != "release" {
        return;
    }

    let exit_status = process::Command::new("yarn")
        .arg("install") // make sure deps are installed
        .arg("prod:build:js")
        .arg("prod:build:css")
        .status()
        .expect("failed to build web app");

    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-changed=src");

    if !exit_status.success() {
        println!("cargo:warning=exit status is {}", exit_status);
        process::exit(1);
    }
}
