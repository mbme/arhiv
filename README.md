# Arhiv
Arhiv is a local-first personal encrypted database. It can store structured records, as well as files.

Arhiv doesn't rely on a central server. Data can be synchronized between multiple devices using solutions like [Syncthing](https://github.com/syncthing/syncthing), or services like Google Drive or Dropbox, or by manually transferring files using USB flash drive etc.
Records are stored in the `baza.gz.age` storage file.
In case of simultaneous edits on multiple devices, there likely to be multiple versions of the storage file.
On startup, Arhiv will merge multiple storage files into one. Conflicts would be automatically resolved; there would be a list of documents with resolved conflicts on UI.

Currently, record schema is hardcoded, so it's impossible to add new record types without recompilation. This will change in future.

There's a cross-platform CLI app that can run a web server with UI. There's Android app, and Electron-based UI (currently it's built only in ArchLinux).

# Privacy and Security
* **All data (including files) is encrypted with [Age encryption](https://age-encryption.org/v1)**.
* An x25519 Age key for storage file & state file is stored in `key.age` in storage dir.
It is encrypted with password-based Age key.
**If you loose this file or your password, you lose access to your data!**.
You should export backup copies of your key using Arhiv CLI or UI.
* Data files have their own Age x25519 keys stored in storage & state.
* Web UI server generates self-signed **HTTPS certificate** and saves it in the state dir **in plain text**.
* Desktop & Android apps verify the server HTTPS certificate.
* Web UI server generates random signed **auth token** on startup.
* Desktop & Android apps
  * start Web UI server and get **auth token** from it
  * send the auth token in a cookie to the Web UI server
  * Web UI server denies requests without the auth token
* In Desktop & CLI apps password is saved to System keyring.
* In Android app password is saved to the System KeyStore.
* Desktop & Android apps **unlock server** using password they got from user or keyring. **The Web UI server stays unlocked** until the app is closed or manually locked.

# Arhiv CLI installation

You can download CLI & Android app builds from [Github Releases](https://github.com/mbme/typed-v/releases).

## Install using Cargo
* `npm install`
* `just cargo-install`

## Install on ArchLinux
Using makepkg: `just arch-install`. It also installs `arhiv-desktop` GUI.

# Build dependencies
* `rust`
* `cargo`
* `nodejs` 23.6+
* `npm`
* `lld` - a fast linker from the LLVM project
* `just` command runner https://github.com/casey/just

# Cross-compiling for Windows
* Add rust target for Windows cross-compilation: `rustup target add x86_64-pc-windows-gnu`
* Install MinGW-w64 toolchain (i.e. `mingw-w64-gcc` on ArchLinux, `mingw-w64` on Ubuntu)
* Run `just prod-build-windows`
* Use `target/x86_64-pc-windows-gnu/release/arhiv.exe`

# Dev tools
* `cargo-outdated` to find out which packages to upgrade
* `cargo-upgrades` to find out which packages to upgrade
* `cargo-machete` to find unused deps
* `cargo-insta` to manage snapshot tests
* `cargo-flamegraph` for performance profiling
* `tmux` for running dev servers in parallel

## Special switches
* `production-mode` feature flag - to distinguish between dev Arhiv & prod Arhiv
* `TYPED_V_VERSION` - env variable to be set on compile time that contains current Arhiv version

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

**Minimum supported Android version is 11(R)**.

**Minimum supported Webview version is 111**.

## Prerequisites
* Android Studio; use it to install Android SDK & NDK
* Android SDK & NDK
* JDK - `jdk-openjdk`
* `cargo-ndk` to build Android JNI library
* Add rust targets for Android cross-compilation (the `x86_64-linux-android` i.e. for Android Studio emulator):
```
rustup target add aarch64-linux-android x86_64-linux-android
```

## Release
* `just prod-build-android-libs prod-build-android-app`
* Install `arhiv.apk`

## Debugging
* Connect your device via USB (ensure Developer mode + USB debugging is enabled)
* Run debug build of Arhiv app on your device (i.e. through Android Studio)
* Open Chrome on your desktop -> `chrome://inspect`
* Find your WebView under Remote Target -> Click Inspect

# Scraper
Arhiv UI supports pasting scraped data from the [Scraper](https://github.com/mbme/scraper) userscript or bookmarklet.