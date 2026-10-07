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

## Commands and validation

Use the most targeted check first, then broaden when needed:

- Rust: `just check-rs`.
- TypeScript/UI/desktop/root linting: `just check-ts`.
- Full local gate: `just check`.
- Markdown: format the intended files with the repository-local `oxfmt`, then validate the same scope with `oxfmt --check`.
- Dev server: `just run`.
- Electron dev runtime: `just desktop`.
- Android/release/package commands live in `justfile`; prefer invoking recipes instead of copying long command lines.

## Required reading by change area

Use `docs/README.md` for navigation and `docs/architecture.md` for system
ownership and runtime flows. Read the relevant contract before changing behavior
in that area:

| Change area                                                                                    | Required reading                                                                              |
| ---------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| Domain concepts, relationships, central business rules                                         | `docs/domain-model.md`                                                                        |
| Owner-facing workflows composing domain and operational rules                                  | `docs/guides/user-workflows.md`                                                               |
| Storage format, encryption, container/index semantics, ordering, patch behavior, file matching | `docs/specs/storage-format.md`, `docs/specs/migration.md`, `docs/runbooks/storage-upgrade.md` |
| Document schema, fields, validation, `data_version`                                            | `docs/specs/schema.md`, `docs/specs/migration.md`, `docs/runbooks/storage-upgrade.md`         |
| Storage migrations, rollback, upgrade preflight and verification                               | `docs/specs/migration.md`, `docs/runbooks/storage-upgrade.md`                                 |
| System-wide security goals, attackers, trust boundaries                                        | `docs/system-threat-model.md`                                                                 |
| Key hierarchy, lifecycle, recoverability                                                       | `docs/specs/key-lifecycle.md`                                                                 |
| Cross-device merge/conflict behavior                                                           | `docs/specs/merge-conflicts.md`                                                               |
| Login, session, token, cookie, certificate behavior                                            | `docs/specs/authentication.md`, `docs/specs/platform-security.md`                             |
| API request/response shapes and Rust/TypeScript compatibility                                  | `docs/specs/api.md`, `arhiv/src/ui/dto.rs`, `arhiv/src/ui/dto.ts`                             |
| Server startup, lockfile, port selection, JSON output, desktop launch                          | `docs/specs/runtime-protocol.md`                                                              |
| Backup, restore, import, status flows                                                          | `docs/specs/backup-restore.md` and relevant `arhiv/src/arhiv/` code                           |
| Desktop or Android platform wrapper security                                                   | `docs/specs/platform-security.md`                                                             |
| Full-text search eligibility, indexing, query matching, ranking, persisted index compatibility | `docs/specs/full-text-search.md`                                                              |
| Significant architecture decisions and rationale                                               | `docs/adr/README.md`                                                                          |

## Safety invariants

- Keep API DTO shapes synchronized between `arhiv/src/ui/dto.rs` and `arhiv/src/ui/dto.ts`.
- Do not change storage container/index semantics (`info` first line, key order, patch behavior) without coordinated storage migration work.
- Preserve the launcher protocol: desktop startup depends on `arhiv server --json` and the `@@SERVER_INFO:` JSON marker on stderr.
- Preserve server single-instance lock and port-discovery semantics in `arhiv/src/server/server_lock.rs` and adjacent startup code.
- Treat certificate generation/trust flow as security-sensitive; desktop and Android verify/pin server cert material.
- Keep Android `minSdk` in `arhiv-android/app/build.gradle` aligned with `android_platform_version` in `justfile`.
- Keep Electron runtime versions aligned across dev and Arch packaging: `arhiv-desktop/package.json`, `package-lock.json`, `PKGBUILD.template`, and `arhiv-desktop/arhiv-desktop` must move together.

## High-risk areas

- Crypto/key handling: `baza-common/src/crypto/`, `baza-storage/src/crypto/`, `baza/src/baza_manager/keys.rs`, `arhiv/src/arhiv/keyring.rs`.
- Storage container and patch/merge logic: `baza-storage/src/container.rs`, `baza/src/baza_storage/`, `baza/src/merge/mod.rs`.
- Auth, cookies, HTTPS, certificates, and launcher trust path: `arhiv/src/server/`, `arhiv-desktop/src/`, `arhiv-android/` WebView/JNI startup code.
- Storage/data migration code: `baza/src/baza_manager/migration/`.
- Android storage/security configuration: `arhiv-android/app/src/main/AndroidManifest.xml`, `arhiv-android/app/src/main/res/xml/backup_rules.xml`, `arhiv-android/app/src/main/res/xml/data_extraction_rules.xml`.

## Security-sensitive validation

- Validate the affected boundary directly rather than relying only on broad checks.
- For server auth changes, exercise missing, malformed, reused, and incorrect credentials as applicable.
- For browser bootstrap changes, verify that the token succeeds once and cannot be reused.
- For Desktop or Android trust changes, verify rejection of certificates other than the startup-delivered certificate.
- For storage crypto or compatibility changes, verify wrong keys, corrupted ciphertext, and incompatible versions fail closed.
- For backup/restore changes, perform recovery with a disposable copy of known committed storage.
- Describe security effects precisely: distinguish confidentiality, integrity, freshness/rollback resistance, availability, and metadata protection.

## Storage migration changes

- Change `storage_version` for persistent container, encoding, encryption/compression, or storage-file merge contract changes.
- Change `data_version` for stored document-shape or meaning changes that invalidate existing data.
- Preserve parseable storage info, exact index/value cardinality, equal `BazaInfo` across mergeable storage files, and every distinct `DocumentKey(id, rev)`.
- Keep migrations deterministic for identical input and target versions, and preserve pre-migration bytes on failure.
- Migrators must be idempotent, hold the exclusive storage lock, log source and target versions, publish through transactional replacement, and fail with actionable diagnostics.
- Cover migration rollback and corruption/failure paths with focused tests.

## Non-obvious gotchas

- Debug and release UI asset serving differ: debug reads filesystem assets, release embeds assets.
- Server binds IPv4 loopback internally and emits localhost URLs in `ServerInfo`.
- Baza storage file matching intentionally accepts sync-conflict filename variants.
- Android requires WebView major version >= 111.
- The desktop packaging wrapper is Linux/Arch-specific; non-Arch desktop packaging is not established here.
- Desktop automated runtime test coverage is effectively unknown; do not assume lint/typecheck exercises desktop startup behavior.

## Project conventions and intent

### Ownership and dependencies

- Keep outbound network/download workflows in `arhiv`; `baza` owns completed local files, encrypted blob storage, schema rules, staging, and other core database behavior.
- Prefer pure-Rust libraries; avoid dependencies that require C/C++ code or native libraries unless explicitly approved.
- Use readable, exact release tags for external GitHub Actions; do not SHA-pin them because SHA pinning is unnecessary for this project.

### Runtime and tooling

- Preserve automatic browser launch for `arhiv server --browser`; development workflows require it.
- Reap browser child processes without coupling their lifetime to the server process.
- Use `just arhiv -- <command>` to inspect configured local debug data when validating CLI behavior, schema, search, document state, histories, or conflicts; start with read-only commands to preserve the fixture.

### Code and documentation

- Add concise comments to new public methods.
- Add high-value doc comments to new or changed public functions, methods, and types, and wherever they clarify ownership, invariants, boundary intent, or caller obligations rather than restating signatures.
- Prefer positive phrasing in code comments and guidance; describe intended ownership and behavior directly rather than framing APIs by what they do not do.
- Keep behavior documentation humane and concise: documents describe behavior rather than source-code layout, while code implements that behavior.
- Add high-value logs around meaningful workflow transitions, security- or data-sensitive decisions, and successful boundary conversions; keep pure helpers quiet and avoid noisy implementation traces.
- Add focused unit tests for durable rules, pure decision logic, boundary behavior, and changed code that owns high-value pure logic, durable rules, or edge-case behavior. Prefer small focused tests over broad incidental coverage and avoid brittle infrastructure.