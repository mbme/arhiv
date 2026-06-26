# Arhiv Agent Guide

## What this repo is
- `arhiv` is a local-first encrypted personal database for structured records and files.
- Runtime surfaces: Rust CLI/server, React UI, Electron desktop wrapper, and Android Java/JNI wrapper.
- Core ownership boundaries:
  - `baza/`: encrypted data model, persistence, schema, merge, search, backup.
  - `baza-storage/`: encrypted storage substrate/container mechanics.
  - `baza-common/`: shared low-level helpers.
  - `arhiv/src/server/`: HTTPS API/UI server, auth/session/cert handling, lockfile/runtime protocol.
  - `arhiv/src/ui/`: React UI and API DTOs.
  - `arhiv-cli/`, `arhiv-desktop/`, `arhiv-android/`: platform entrypoints.

## Canonical specs
Read the relevant spec before changing behavior in that area:
- Storage format/container/index semantics: `docs/arhiv-encrypted-file-format.md`.
- Schema/data-version contract: `docs/storage-schema-contract-spec.md`.
- Storage migrations/rollback rules: `docs/storage-migration-playbook.md`.
- Key hierarchy/lifecycle/recoverability: `docs/crypto-key-lifecycle-threat-model.md`.
- Cross-device conflict semantics: `docs/merge-conflicts-spec.md`.
- Auth/session/certificate trust chain: `docs/auth-session-trust-chain-spec.md`.
- Rust/TypeScript API DTO compatibility: `docs/api-dto-contract-spec.md`.
- Launcher/server startup protocol: `docs/launcher-server-runtime-protocol-spec.md`.
- Backup/restore safety guarantees: `docs/backup-restore-durability-spec.md`.
- Platform trust boundaries: `docs/platform-security-boundaries-spec.md`.

## Safety invariants
- Keep API DTO shapes synchronized between `arhiv/src/ui/dto.rs` and `arhiv/src/ui/dto.ts`.
- Do not change storage container/index semantics (`info` first line, key order, patch behavior) without coordinated storage migration work.
- Preserve the launcher protocol: desktop startup depends on `arhiv server --json` and the `@@SERVER_INFO:` JSON marker on stderr.
- Preserve server single-instance lock and port-discovery semantics in `arhiv/src/server/server_lock.rs` and adjacent startup code.
- Treat certificate generation/trust flow as security-sensitive; desktop and Android verify/pin server cert material.
- Keep Android `minSdk` in `arhiv-android/app/build.gradle` aligned with `android_platform_version` in `justfile`.
- Keep Electron runtime versions aligned across dev and Arch packaging: `arhiv-desktop/package.json`, `package-lock.json`, `PKGBUILD.template`, and `arhiv-desktop/arhiv-desktop` must move together.

## High-risk areas
- Crypto/key handling: `baza-common/src/crypto/`, `baza-storage/src/crypto/`, `arhiv/src/support/crypto_key.rs`, `baza/src/baza_manager/keys.rs`.
- Storage container and patch/merge logic: `baza-storage/src/container.rs`, `baza/src/baza_storage/`, `baza/src/merge.rs`.
- Auth, cookies, HTTPS, certificates, and launcher trust path: `arhiv/src/server/`, `arhiv-desktop/src/`, `arhiv-android/` WebView/JNI startup code.
- Dangerous migration helpers: `baza/src/baza_manager/migration.rs` methods prefixed with `dangerously_`.
- Android storage/network security configuration: `arhiv-android/app/src/main/AndroidManifest.xml`, `arhiv-android/app/src/main/res/xml/network_security_config.xml`.

## Before changing X, read Y
- Storage format, encryption, container ordering, or file matching -> encrypted file format spec + migration playbook.
- Document schema, fields, validation, or `data_version` -> storage schema contract + migration playbook.
- API request/response shape -> API DTO contract + both Rust/TS DTO files.
- Server startup, lockfile, port selection, JSON output, desktop launch -> launcher/server runtime protocol spec.
- Login/session/token/cookie/cert behavior -> auth-session trust-chain spec + platform security boundaries spec.
- Backup/restore/import/status flows -> backup/restore durability spec and relevant `arhiv/src/arhiv/` code.
- Cross-device merge/conflict behavior -> merge conflicts spec.
- Desktop or Android platform wrapper security -> platform security boundaries spec.

## Common validation commands
Use the most targeted check first, then broaden when needed:
- Rust: `just check-rs`.
- TypeScript/UI/desktop/root linting: `just check-ts`.
- Full local gate: `just check`.
- Dev server: `just run`.
- Electron dev runtime: `just desktop`.
- Android/release/package commands live in `justfile`; prefer invoking recipes instead of copying long command lines.

## Non-obvious gotchas
- Debug and release UI asset serving differ: debug reads filesystem assets, release embeds assets.
- Server binds `0.0.0.0` internally but emits localhost URLs in `ServerInfo`.
- Baza storage file matching intentionally accepts sync-conflict filename variants.
- Android requires WebView major version >= 111.
- The desktop packaging wrapper is Linux/Arch-specific; non-Arch desktop packaging is not established here.
- Desktop automated runtime test coverage is effectively unknown; do not assume lint/typecheck exercises desktop startup behavior.
