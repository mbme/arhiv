use std::{env, process};

// build app in release mode
fn main() {
    if env::var("PROFILE").unwrap() != "release" {
        return;
    }

    // make sure deps are installed
    let install_status = process::Command::new("yarn")
        .arg("install")
        .status()
        .expect("failed to install yarn deps");
    if !install_status.success() {
        println!(
            "cargo:warning=yarn install exit status is {}",
            install_status
        );
        process::exit(1);
    }

    // run build
    let build_status = process::Command::new("yarn")
        .arg("prod:build")
        .status()
        .expect("failed to run build");
    if !build_status.success() {
        println!("cargo:warning=yarn build exit status is {}", build_status);
        process::exit(2);
    }
}
