# Arhiv

Arhiv is a local-first personal encrypted database. It can store structured records, as well as files.

Arhiv doesn't rely on a central server. Data can be synchronized between multiple devices using solutions like [Syncthing](https://github.com/syncthing/syncthing), or services like Google Drive or Dropbox, or by manually transferring files using USB flash drive etc.
Records are stored in the `baza.gz.age` storage file.
In case of simultaneous edits on multiple devices, there likely to be multiple versions of the storage file.
On startup, Arhiv will merge multiple storage files into one. Conflicts would be automatically resolved; there would be a list of documents with resolved conflicts on UI.

Currently, record schema is hardcoded, so it's impossible to add new record types without recompilation. This will change in future.

There's a cross-platform CLI app that can run a web server with UI. There's Android app, and Electron-based UI (currently it's built only in ArchLinux).

# Privacy and Security

- **All data (including files) is encrypted with [Age encryption](https://age-encryption.org/v1)**.
- An x25519 Age key for storage file & state file is stored in `key.age` in storage dir.
  It is encrypted with password-based Age key.
  **If you loose this file or your password, you lose access to your data!**.
  You should export backup copies of your key using Arhiv CLI or UI.
- Data files have their own Age x25519 keys stored in storage & state.
- Web UI server generates self-signed **HTTPS certificate** and saves it in the state dir **in plain text**.
- Desktop & Android apps verify the server HTTPS certificate.
- Web UI server generates random signed **auth token** on startup.
- Desktop & Android apps
  - start Web UI server and get **auth token** from it
  - send the auth token in a cookie to the Web UI server
  - Web UI server denies requests without the auth token
- In Desktop & CLI apps password is saved to System keyring.
- In Android app password is saved to the System KeyStore.
- Desktop & Android apps **unlock server** using password they got from user or keyring. **The Web UI server stays unlocked** until the app is closed or manually locked.

# Usage with Syncthing

When using Syncthing to synchronize Arhiv storage across devices, you should turn off the file versioning (it seems to be turned on by default on Android).
That way, if you'll modify Arhiv on both devices simultaneously, Syncthing will keep both storage file versions, and Arhiv will merge them after restart.

# Specification docs

- `docs/domain-model.md`: business concepts, relationships, and central rules for the Arhiv.
- `docs/arhiv-encrypted-file-format.md`: canonical on-disk encrypted format, container/index invariants, and compatibility boundaries.
- `docs/storage-schema-contract-spec.md`: document-type/field schema contract, runtime validation behavior, and `data_version` migration triggers.
- `docs/storage-migration-playbook.md`: operational migration procedure, rollback rules, and non-negotiable migration invariants.
- `docs/crypto-key-lifecycle-threat-model.md`: key hierarchy, lifecycle operations, recoverability constraints, and security assumptions.
- `docs/merge-conflicts-spec.md`: cross-device conflict detection/resolution semantics and idempotency guarantees.
- `docs/auth-session-trust-chain-spec.md`: auth token/session model plus desktop/android certificate trust-chain contract.
- `docs/api-dto-contract-spec.md`: Rust/TypeScript DTO and `/ui/api` request/response compatibility contract.
- `docs/launcher-server-runtime-protocol-spec.md`: launcher-to-server startup protocol (`--json`, `@@SERVER_INFO`), lock/port/shutdown semantics.
- `docs/backup-restore-durability-spec.md`: backup/restore scope, safety guarantees, and corruption handling expectations.
- `docs/platform-security-boundaries-spec.md`: desktop/android/server trust boundaries, secret handling surfaces, and non-goals.
- `docs/full-text-search-spec.md`: full-text search indexing scope, strict AND eligibility, candidate matching, ranking, and index compatibility rules.

# Arhiv CLI installation

You can download CLI & Android app builds from [Github Releases](https://github.com/mbme/arhiv/releases).

## Install using Cargo

- `npm install`
- `just cargo-install`

## Install on ArchLinux

Using makepkg: `just arch-install`. It also installs `arhiv-desktop` GUI.

# Build dependencies

- `rust`
- `cargo`
- `nodejs` 23.6+
- `npm`
- `lld` - a fast linker from the LLVM project
- `just` command runner https://github.com/casey/just

# Cross-compiling for Windows

- Add rust target for Windows cross-compilation: `rustup target add x86_64-pc-windows-gnu`
- Install MinGW-w64 toolchain (i.e. `mingw-w64-gcc` on ArchLinux, `mingw-w64` on Ubuntu)
- Run `just prod-build-windows`
- Use `target/x86_64-pc-windows-gnu/release/arhiv.exe`

# Dev tools

- `cargo-outdated` to find out which packages to upgrade
- `cargo-upgrades` to find out which packages to upgrade
- `cargo-machete` to find unused deps
- `cargo-insta` to manage snapshot tests
- `cargo-flamegraph` for performance profiling
- `tmux` for running dev servers in parallel

## Special switches

- `production-mode` feature flag - to distinguish between dev Arhiv & prod Arhiv
- `ARHIV_VERSION` - env variable to be set on compile time that contains current Arhiv version

## Release process

- Releases use plain numeric Git tags. Run `just bump-version` to create and push the next tag.
- A pushed tag runs the GitHub release workflow. It publishes the Linux CLI binary, Windows CLI binary, and signed Android APK to GitHub Releases.
- Android releases are distributed as APKs for sideloading; there is no Play Store publishing process.
- The Electron desktop app is not published by the release workflow. On Arch Linux, `just arch-install` builds and installs the package locally, including `arhiv-desktop`.

# CLI app

Cross-platform CLI app. Uses system keyring to store password.

Useful document commands:

- `arhiv list` lists recent documents; use `--type`, `--page`, `--conflicts`, or `--json` to narrow output.
- `arhiv search <query>` searches documents with the same filtering/output options as `list`.
- `arhiv get <id>` prints a readable document summary; add `--json` for the raw document head.
- `arhiv conflicts` lists conflicted documents, and `arhiv conflict show <id>` prints conflict branches plus any staged resolution.
- `arhiv history <id>` lists committed snapshots, `arhiv snapshot get <id> <rev>` prints one snapshot, and `arhiv revert <id> <rev>` stages a historical snapshot as current data.
- `arhiv diff staged|snapshots|conflict ...` prints unified diffs of canonical document JSON data.
- `arhiv reset <id>` discards a staged document change or conflict resolution; `arhiv reset --all` discards all staged changes.
- `arhiv add <type> <json>`, `arhiv update <id> <json>`, and `arhiv erase <id>` manage document data.
- `arhiv import track <file...>` imports audio files as track documents.
- `arhiv schema [type]` prints available document types or a type's fields.
- `arhiv collection list <id>` lists collections containing a document.
- `arhiv collection members <collection-id>` lists ordered collection members.
- `arhiv collection add|remove|move <collection-id> <id>` updates collection membership.
- `arhiv asset create <file...>` creates encrypted asset documents from local files.
- `arhiv asset export <id> <output-file>` decrypts an asset into a local file.

# Web UI app

- `TypeScript` for type checking
- `Oxlint` for linting
- `Oxfmt` for code formatting
- `React.js` for rendering
- `TailwindCSS` for styling
- `esbuild` for bundling the app

# Desktop app

Cross-platform desktop app that uses `Electron` to display Web UI. Uses system keyring to store password.

# Android app

Java Webview app that displays Web UI. Uses biometric authentication or device authentication to safely store password in KeyStore.
Needs `MANAGE_EXTERNAL_STORAGE` permission to read/write files in user directory (next to Music, Downloads etc.).

**Minimum supported Android version is 11(R)**.

**Minimum supported Webview version is 111**.

## Prerequisites

- Android Studio; use it to install Android SDK & NDK
- Android SDK & NDK
- JDK - `jdk-openjdk`
- generate keystore
- create `arhiv-android/keystore.properties`
- `cargo-ndk` to build Android JNI library
- Add rust targets for Android cross-compilation (the `x86_64-linux-android` i.e. for Android Studio emulator):

```
rustup target add aarch64-linux-android x86_64-linux-android
```

## Release

- Generate keystore (it's mandatory to sign the release apk):

```
keytool -genkeypair \
  -alias Arhiv \
  -keyalg RSA -keysize 2048 \
  -validity 10000 \
  -keystore release.keystore \
  -dname "CN=Your Name, OU=Your Org, O=Your Company, L=City, ST=State, C=US" \
  -storepass YOUR_STORE_PASS \
  -keypass YOUR_KEY_PASS
```

- Put the `release.keystore` into arhiv-android dir.
- In the arhiv-android dir, create `keystore.properties`:

```
storeFile=../release.keystore
storePassword=YOUR_STORE_PASS
keyAlias=Arhiv
keyPassword=YOUR_KEY_PASS
```

- `just prod-build-android-libs prod-build-android-app`
- Install `arhiv.apk`

## Debugging

- Connect your device via USB (ensure Developer mode + USB debugging is enabled)
- Run debug build of Arhiv app on your device (i.e. through Android Studio)
- Open Chrome on your desktop -> `chrome://inspect`
- Find your WebView under Remote Target -> Click Inspect

# Scraper

Arhiv UI supports pasting scraped data from the [Scraper](https://github.com/mbme/scraper) userscript or bookmarklet.

# TODO

- check if can use https://doc.rust-lang.org/std/fs/struct.File.html#method.lock instead of 3rd party
- check if can improve multi-file fstransaction
- better search - vector search? integrate tantivy?
- better diff/merge
- network p2p sync (use iroh), relays
- integration with emacs/file system - use FUSE?
- get rid of tailwind?
- domain docs to cover user workflows?
- tool, not app

- як це розкладаєтьс на "базові" компоненти? і інтегрується із рештою екосистеми?
  - формат даних (encrypted compressed jsonl)
  - проста база даних із схемою, гілками і мерджем, eventual consistency & conflict resolution
  - p2p sync
  - arhiv app
  - storage/(read/write APIs i.e. FS) for other apps
- UI: i don't like switching between edit/preview modes in the editor
- single folder mode? keep state in "syncable dir"?
- refactor: arhiv-cli shouldn't probably access baza directly
- optimize storage compression (snapshot order)
- remote backup without 3rd party tools - separate "backup manager"?
- mark notes stale/irrelevant/archived
- UI improve password input: allow to see plain text