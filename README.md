# Installation

## Using cargo
* `just cargo-install`

## On ArchLinux, using makepkg
* `just prod-build-install`

# Build dependencies
* `rust`
* `cargo`
* `nodejs` 23.6+
* `npm`
* `lld` - a fast linker from the LLVM project
* `just` command runner https://github.com/casey/just

# Dev tools
* `cargo-outdated` to find out which packages to upgrade
* `cargo-upgrades` to find out which packages to upgrade
* `cargo-machete` to find unused deps
* `cargo-insta` to manage snapshot tests
* `cargo-flamegraph` for performance profiling
* `tmux` for running dev servers in parallel

## Special switches
* `production-mode` feature flag

## Release process
* `just bump-version` - increment major version, create & push git tag

# Web UI app
* Typescript for type checking
* Eslint for linting
* Prettier for code formatting
* React for rendering
* TailwindCSS for styling
* Esbuild for bundling the app

# Android app

## Prerequisites
* Android Studio; use it to install Android SDK & NDK
* `cargo-ndk` to build Android JNI library
* Add rust targets for Android cross-compilation:
```
rustup target add aarch64-linux-android x86_64-linux-android
```
* JDK - `jdk-openjdk`
* Android SDK & NDK

# Scraper
Arhiv UI supports pasting scraped data from the [Scraper](https://github.com/mbme/scraper) userscript or bookmarklet.