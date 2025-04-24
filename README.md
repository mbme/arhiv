# Privacy and Security
* All the data is encrypted with Age encryption.
* An x25519 Age key for storage file & state file is stored in key.age in storage dir. It is encrypted with password-based Age key.
* Data files have their own Age x25519 keys stored in storage & state.
* Web UI server generates self-signed **HTTPS certificate** and saves it in the state dir **in plain text**.
* Desktop & Android apps verify the server HTTPS certificate.
* Web UI server generates random signed **auth token** on startup.
* Desktop & Android apps
  * start Web UI server and get **auth token** from it
  * send the auth token in a cookie to the Web UI server
  * Web UI server denies requests without the auth token
* In Desktop & CLI apps user can save password to System keyring.
* In Android app user can save password to the System KeyStore.
* Desktop & Android apps **unlock server** using password they got from user or keyring. **The Web UI server stays unlocked** until the app is closed or manually locked.

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
Java Webview app that displays Web UI. Uses biometric authentication or device authentication to safely store password in KeyStore.
Needs `MANAGE_EXTERNAL_STORAGE` permission to read/write files in user directory (next to Music, Downloads etc.).

## Prerequisites
* Android Studio; use it to install Android SDK & NDK
* Android SDK & NDK
* JDK - `jdk-openjdk`
* `cargo-ndk` to build Android JNI library
* Add rust targets for Android cross-compilation (the `x86_64-linux-android` i.e. for Android Studio emulator):
```
rustup target add aarch64-linux-android x86_64-linux-android
```

# Scraper
Arhiv UI supports pasting scraped data from the [Scraper](https://github.com/mbme/scraper) userscript or bookmarklet.