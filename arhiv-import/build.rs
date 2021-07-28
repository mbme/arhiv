use std::{env, process};

fn main() {
    let is_release = env::var("PROFILE").unwrap_or_default() == "release";

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
        .arg({
            if is_release {
                "prod:build"
            } else {
                "build"
            }
        })
        .status()
        .expect("failed to run build");

    if !build_status.success() {
        println!("cargo:warning=yarn build exit status is {}", build_status);
        process::exit(2);
    }
}
