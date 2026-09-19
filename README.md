# Arhiv

Arhiv is a local-first encrypted personal database for structured records and
files. It runs without a central server and supports a CLI, local web UI,
Electron desktop wrapper, and Android application.

Record schemas are compiled into the application. Arhiv stores committed record
snapshots in `baza.gz.age` and can synchronize storage through file-sync tools
such as Syncthing, cloud drives, or manual transfer.

When synchronized devices produce multiple storage files, Arhiv preserves their
distinct snapshots and detects concurrent revisions. It prepares a heuristic
merged version for review; the record remains in conflict until that version is
committed.

## Privacy and security

- Record and attachment contents are encrypted at rest with [age](https://age-encryption.org/v1). Filesystem metadata such as names, sizes, timestamps, and directory layout is not concealed.
- The x25519 storage master key is stored in `key.age`, encrypted by a password-derived age key.
- Losing every usable key copy and the passwords needed to decrypt them makes the data unrecoverable. Keep protected key exports together with tested storage backups.
- Attachments use individual x25519 keys stored in their encrypted record metadata.
- The Web UI server uses a persistent self-signed HTTPS certificate and a random opaque authentication token generated at startup.
- Desktop and Android pin the certificate delivered through their local startup boundary and establish the authenticated cookie directly.
- Desktop can cache the serialized storage master key in the system keyring. Android protects its cached key with an authentication-gated Android Keystore key.
- Storage remains unlocked in the running server until the application closes or the owner locks it.

See the [system threat model](docs/system-threat-model.md) for the complete
security boundaries and non-guarantees.

## Applications

- **CLI and local web UI:** cross-platform Rust application. Run `arhiv --help` for available commands.
- **Desktop:** Electron wrapper around the local web UI. Local packaging is currently established for Arch Linux.
- **Android:** Java WebView wrapper with a Rust/JNI server. It requires Android 11 or newer and WebView major version 111 or newer.

Arhiv also accepts pasted data produced by the
[Scraper](https://github.com/mbme/scraper) userscript or bookmarklet.

## Installation

CLI and Android builds are available from
[GitHub Releases](https://github.com/mbme/arhiv/releases).

Install the CLI from this checkout:

```sh
npm install
just cargo-install
```

On Arch Linux, `just arch-install` builds and installs the CLI and desktop
wrapper.

## Synchronization with Syncthing

When devices modify the same storage file concurrently, Syncthing may retain
one version as a `sync-conflict` copy. Arhiv recognizes these conflict filenames
and merges their distinct snapshots on the next open.

Syncthing file versioning is separate from conflict-copy handling. Configure it
according to your recovery needs; Arhiv does not require it to be disabled.

## Documentation

The [documentation guide](docs/README.md) links the current domain, storage,
synchronization, security, runtime, and interface documentation.

Development setup, checks, packaging, and release details are in
[CONTRIBUTING.md](CONTRIBUTING.md). Android-specific build and debugging
instructions are in [arhiv-android/README.md](arhiv-android/README.md).
