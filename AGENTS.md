# Arhiv Agent Guide

## One-minute overview
- `arhiv` is a local-first encrypted personal database for structured records and files (`README.md`).
- Core runtime is the Rust CLI binary at `binutils/src/bin/arhiv.rs`; it can initialize storage, manage keys/passwords, and run the HTTPS UI server.
- The UI server is in `arhiv/src/server/` (Axum + Rustls) and serves the React UI from `arhiv/public/` in debug or embedded assets in release (`arhiv/src/server/ui_server/public_assets_handler.rs`).
- Data model/persistence is in `baza/`; cross-cutting crypto/container/fs/http utilities are in `rs-utils/`.
- Platform wrappers: Electron (`arhiv-desktop/`) and Android Java + JNI (`arhiv-android/`).

## Authoritative directory map
- `.github/actions/setup/action.yml`: shared CI setup (Rust toolchain, Node 23.7, `npm ci`, system packages).
- `.github/workflows/check.yml`: CI checks on `master` push/PR.
- `.github/workflows/release.yml`: tag-triggered Linux/Windows CLI + Android APK build and GitHub release publish.
- `.cargo/config.toml`: linker/rustflags (`lld`, native CPU on Linux, static CRT on Windows GNU).
- `Cargo.toml`: Rust workspace members: `baza`, `arhiv`, `binutils`, `rs-utils`, `arhiv-android`.
- `package.json`: npm workspaces: `arhiv`, `arhiv-desktop`.
- `justfile`: canonical task entrypoints for dev/build/check/release.
- `baza/src/`: encrypted storage/state management, schema, merge, search state, backup.
- `arhiv/src/arhiv/`: app lifecycle, keyring integration, import/status.
- `arhiv/src/server/`: HTTPS server, lockfile, certificate handling, HTTP API/UI routes.
- `arhiv/src/ui/`: React/TypeScript UI, DTOs (`dto.ts`) mirrored with Rust DTOs (`dto.rs`).
- `binutils/src/bin/arhiv.rs`: CLI entrypoint and subcommands.
- `rs-utils/src/`: reusable primitives (age crypto, container format, HTTP server, fs, logging).
- `arhiv-desktop/src/index.ts`: Electron entrypoint; starts CLI server and opens window.
- `arhiv-android/app/src/main/java/me/mbsoftware/arhiv/MainActivity.java`: Android app entrypoint and WebView host.
- `arhiv-android/src/lib.rs`: Rust JNI bridge (`Java_me_mbsoftware_arhiv_ArhivServer_*`).
- `resources/`: sample/static assets; exact runtime role is unknown.

## Entry points and execution surfaces
- CLI binary: `binutils/src/bin/arhiv.rs` (`fn main()` with `clap` subcommands).
- Server runtime: CLI subcommand `Server` in `binutils/src/bin/arhiv.rs`, implemented by `ArhivServer::start` in `arhiv/src/server/mod.rs`.
- Desktop runtime: `arhiv-desktop/src/index.ts` -> `startServer()` in `arhiv-desktop/src/arhiv.ts` -> spawns `arhiv server --json`.
- Android runtime: `MainActivity` calls `ArhivServer.startServer(...)` (`arhiv-android/app/src/main/java/me/mbsoftware/arhiv/ArhivServer.java`) -> JNI exports in `arhiv-android/src/lib.rs`.
- Library crates: `baza/src/lib.rs`, `arhiv/src/lib.rs`, `rs-utils/src/lib.rs`.

## Run/build/test/lint commands (verbatim)
- `npm install` (`README.md`).
- `cargo run -p binutils --bin arhiv server` (`README.md`).
- `npm run build --workspace arhiv` (`README.md`, `justfile`).
- `npm run check` (`package.json`, `justfile`).
- `just check-rs` (`.github/workflows/check.yml`).
- `just check-ts` (`.github/workflows/check.yml`).
- `just prod-build` (`.github/workflows/release.yml`).
- `just prod-build-windows` (`README.md`, `.github/workflows/release.yml`).
- `just prod-build-android-libs` (`.github/workflows/release.yml`).
- `just prod-build-android-app` (`README.md`, `.github/workflows/release.yml`).
- `just cargo-install` (`README.md`).
- `just arch-install` (`README.md`).
- `cargo clippy --all-targets --all-features -- -D warnings` (`justfile`).
- `cargo test` (`justfile`).
- `cd arhiv-android; ./gradlew assembleDebug` (`justfile`).
- `./gradlew assembleRelease --no-daemon` (`justfile`).
- `cd arhiv-android; ./gradlew installDebug` (`justfile`).
- `adb install -r arhiv.apk` (`justfile`).
- `cargo ndk -t arm64-v8a --platform {{android_platform_version}} -o ./app/src/main/jniLibs build --frozen --release --features production-mode` (`justfile`).
- Additional run/build targets exist in `justfile`: `run`, `desktop`, `check`, `build-android-libs`, `prod-build-desktop`, `install-android-app`, `install-prod-android-app`.

## Tests and quality gates
- Rust: `cargo clippy --all-targets --all-features -- -D warnings` and `cargo test` (`just check-rs`).
- TS: root `npm run check` runs `compiler-errors`, `check-fmt`, `lint`, `test` across workspaces (`package.json`).
- Frontend tests are in `arhiv/src/ui/**/*.test.ts` and `arhiv/src/ui/**/*.test.tsx`; command is the `test` script in `arhiv/package.json`.
- Rust tests are mostly inline `#[cfg(test)]` module tests across `baza/src/` and `rs-utils/src/`, plus integration tests in `rs-utils/tests/container.rs`.
- Desktop test script exists but is empty (`arhiv-desktop/package.json` -> `"test": ""`); desktop runtime test coverage in CI is effectively unknown.

## Required env vars and config files
- `DEV_ARHIV_ROOT`: required in dev mode (`arhiv/src/arhiv/mod.rs`), used by `run` and `desktop` recipes in `justfile`.
- `SERVER_PORT`: CLI server arg env fallback (`binutils/src/bin/arhiv.rs`).
- `BROWSER`: required when running server with `--browser` (`binutils/src/bin/arhiv.rs`).
- `RUST_LOG`: consumed by tracing env filter (`rs-utils/src/log.rs`), set in `run`/`desktop` recipes in `justfile`.
- `ARHIV_VERSION`: compile-time version (`rs-utils/src/lib.rs`), injected in production build recipes (`justfile`) and Android Gradle.
- `ARHIV_BIN`: required for Electron dev mode (`arhiv-desktop/src/arhiv.ts`), set by the `desktop` recipe in `justfile`.
- `NODE_ENV`: controls JS bundling mode in `arhiv/build.ts` and `arhiv-desktop/build.ts`.
- Android release config file: `arhiv-android/keystore.properties` is required by `arhiv-android/app/build.gradle`.
- Android env vars in Gradle: `VERSION_CODE`, `ARHIV_VERSION` (`arhiv-android/app/build.gradle`).

## CI/CD and release flow
- Check pipeline: `.github/workflows/check.yml` runs `just check-rs` and `just check-ts`.
- Release pipeline: `.github/workflows/release.yml` runs on tags and manual dispatch; builds Linux CLI, Windows CLI, Android libs/APK, then publishes artifacts with `ncipollo/release-action`.
- Release workflow dependencies include cargo target installs, `cargo-ndk`, Android SDK, JDK 17, and signing secrets (`RELEASE_KEYSTORE_BASE64`, `KEY_STORE_PASS`, `KEY_ALIAS`, `KEY_PASS`).

## External integrations
- OS keyring integration in Rust (`keyring` crate) via `arhiv/src/arhiv/keyring.rs` and CLI flows.
- Android secure storage via KeyStore + BiometricPrompt in `arhiv-android/app/src/main/java/me/mbsoftware/arhiv/Keyring.java`.
- Scraper integration via npm dependency `github:mbme/scraper#semver:v2.11.0` (`arhiv/package.json`).
- Optional sync tools mentioned in docs (Syncthing/Drive/Dropbox) are user-level operational choices (`README.md`), not runtime service dependencies.

## High-risk/sensitive areas
- Crypto primitives and key handling: `rs-utils/src/crypto/`, `baza/src/baza_manager/keys.rs`.
- Storage container format and patch/merge logic: `rs-utils/src/container.rs`, `baza/src/baza_storage/`.
- Server auth/cookie and HTTPS certificate trust path: `arhiv/src/server/`, `arhiv/src/server/ui_server/mod.rs`, `arhiv-desktop/src/index.ts`, Android `MainActivity.java`.
- Migration helpers marked dangerous: `baza/src/baza_manager/migration.rs` (`dangerously_*` methods).
- Android broad storage permission and custom network security config: `arhiv-android/app/src/main/AndroidManifest.xml`, `arhiv-android/app/src/main/res/xml/network_security_config.xml`.

## Change-safety rules inferred from codebase
- Keep API DTO shapes synchronized between `arhiv/src/ui/dto.rs` and `arhiv/src/ui/dto.ts`.
- Do not change container/index semantics (`info` first line, key order, patch behavior) without coordinated storage migration updates (`baza/src/baza_storage/`, `rs-utils/src/container.rs`).
- Preserve server info contract (`@@SERVER_INFO:` JSON marker on stderr) used by desktop startup (`arhiv-desktop/src/arhiv.ts`).
- Keep Android `minSdk` in `arhiv-android/app/build.gradle` aligned with `android_platform_version` in `justfile` (explicit warning in both).
- Preserve single-instance lock semantics (`arhiv/src/server/server_lock.rs`) when touching server startup/port discovery.
- Be careful changing certificate generation/trust flow; both desktop and Android pin/verify server cert material.

## Non-obvious design decisions / gotchas
- Debug vs release static assets differ: filesystem reads in debug, embedded assets in release (`arhiv/src/server/ui_server/public_assets_handler.rs`).
- Server binds `0.0.0.0` internally (`rs-utils/src/http_server.rs`) but emits localhost URLs in `ServerInfo` (`arhiv/src/server/server_info.rs`).
- Baza storage file matching intentionally accepts sync-conflict filename variants (`baza/src/baza_paths.rs::is_baza_file`).
- Android app requires WebView major version >= 111 (`MainActivity.ensureMinWebViewVersion`).
- Desktop packaging wrapper script is Linux/Arch-specific (`arhiv-desktop/arhiv-desktop`); non-Arch packaging flow is unknown.

## Unknowns
- Dedicated production deployment topology (beyond CLI/Desktop/Android local runtime) is unknown.
- Formal storage schema migration policy for breaking format changes is unknown.
- Desktop automated test strategy beyond lint/typecheck is unknown.
