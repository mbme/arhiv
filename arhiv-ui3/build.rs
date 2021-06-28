use std::{env, process};

// build web app in release mode
fn main() {
    if env::var("PROFILE").unwrap() != "release" {
        return;
    }

    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-changed=src");

    let install_status = process::Command::new("yarn")
        .arg("install") // make sure deps are installed
        .status()
        .expect("failed to install yarn deps");
    if !install_status.success() {
        println!(
            "cargo:warning=yarn install exit status is {}",
            install_status
        );
        process::exit(1);
    }

    let build_status = process::Command::new("yarn")
        .arg("prod:build:js")
        .arg("prod:build:css")
        .status()
        .expect("failed to build web app");

    if !build_status.success() {
        println!("cargo:warning=yarn build exit status is {}", build_status);
        process::exit(2);
    }
}
