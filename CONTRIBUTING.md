# Contributing to Arhiv

## Development environment

The local and CI builds use:

- Rust and Cargo;
- Node.js 24 and npm;
- `just` as the command runner;
- LLD as the Rust linker; and
- the DBus development library on Linux for system-keyring support.

Install JavaScript dependencies from the repository root:

```sh
npm install
```

## Checks and local development

Run the full local check:

```sh
just check
```

Focused checks are available as `just check-rs` and `just check-ts`.

Start the development server and UI watchers with:

```sh
just run
```

This recipe uses `tmux`. Start the Electron wrapper with `just desktop`.

The `production-mode` Cargo feature separates production data and credentials
from development state. Production builds receive their displayed version
through the `ARHIV_VERSION` environment variable.

## Building and installing

Install a production CLI build locally:

```sh
just cargo-install
```

On Arch Linux, build and install the CLI and Electron package with:

```sh
just arch-install
```

Desktop packaging outside Arch Linux is not currently established.

### Cross-compiling the CLI for Windows

Install the Rust target and a MinGW-w64 toolchain, then run the build recipe:

```sh
rustup target add x86_64-pc-windows-gnu
just prod-build-windows
```

The executable is written to
`target/x86_64-pc-windows-gnu/release/arhiv.exe`.

### Android

See [arhiv-android/README.md](arhiv-android/README.md) for Android SDK, signing,
build, installation, and WebView debugging instructions.

## Release process

Releases use plain numeric Git tags. `just bump-version` creates and pushes the
next tag after confirmation.

A pushed tag runs `.github/workflows/release.yml`, which publishes:

- the Linux CLI binary;
- the Windows CLI binary; and
- the signed Android APK; and
- the combined license text (also bundled in the APK).

The Android APK is distributed for sideloading. The Electron desktop wrapper is
not published by the release workflow.

## Optional development tools

The project occasionally uses `cargo-outdated`, `cargo-upgrades`,
`cargo-machete`, `cargo-insta`, and `cargo-flamegraph` for dependency review,
snapshot management, and profiling. They are not required for the normal check
or build recipes.