# Installation

## Using cargo
* `just cargo-install`

## On ArchLinux, using makepkg
* `just prod-build-install`
* `sudo systemctl --user daemon-reload`
* `sudo systemctl --user enable --now arhiv-service`

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
* `cargo-flamegraph` for performance profiling
* `tmux` for running dev servers in parallel

## Special switches
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

# Scraper
Arhiv UI supports pasting scraped data from the [Scraper](https://github.com/mbme/scraper) userscript or bookmarklet.

