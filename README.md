# Installation

## Using cargo
* `just cargo-install`

## On ArchLinux, using makepkg
* `just prod-build-install`
* `sudo systemctl daemon-reload`
* `sudo systemctl enable --now arhiv-service@<username>`

# Build dependencies
* `rust`
* `cargo`
* `nodejs`
* `npm`
* `lld` - a fast linker from the LLVM project
* `just` command runner https://github.com/casey/just

# Dev dependencies
* `cargo-outdated` to find out which packages to upgrade
* `cargo-upgrades` to find out which packages to upgrade
* `cargo-insta` to manage snapshot tests
* `tmux` for running dev servers in parallel

## Special switches
* `JSON_ARG_MOODE` env variable for some CLIs allows to receive arguments as a JSON object
* `production-mode` feature flag

# Android app

## Prerequisites
* `cargo-ndk` to build android JNI libraries
* JDK - `jdk-openjdk`
* Android SDK & NDK
* Add rust targets for Android: 
```
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android
```

# ARCHITECTURE