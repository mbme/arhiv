# Installation

## Using Cargo
* `npm install`
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
* `production-mode` feature flag - to distinguish between dev Arhiv & prod Arhiv

## Release process
* `just bump-version` - increment major version, create & push git tag

# CLI app
Cross-platform CLI app. Uses system keyring to store password.

# Web UI app
* `TypeScript` for type checking
* `ESLint` for linting
* `Prettier` for code formatting
* `React.js` for rendering
* `TailwindCSS` for styling
* `esbuild` for bundling the app

# Desktop app
Cross-platform desktop app that uses `Electron` to display Web UI. Uses system keyring to store password.

# Android app
Java Webview app that displays Web UI. Uses biometric authentication or device authentication to safely store password.
Needs `MANAGE_EXTERNAL_STORAGE` permission to read/write files in user directory (next to Music, Downloads etc.).

## Prerequisites
* Android Studio; use it to install Android SDK & NDK
* Android SDK & NDK
* JDK - `jdk-openjdk`
* `cargo-ndk` to build Android JNI library
* Add rust targets for Android cross-compilation (the `x86_64-linux-android` for Android Studio emulator):
```
rustup target add aarch64-linux-android x86_64-linux-android
```

# Scraper
Arhiv UI supports pasting scraped data from the [Scraper](https://github.com/mbme/scraper) userscript or bookmarklet.