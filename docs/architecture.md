# Architecture

Arhiv is one owner's local-first encrypted database. A Rust core serves the CLI
and a local HTTPS UI; Electron and Android wrap that UI. This overview describes
the supported system, not the [draft P2P design](proposals/p2p-sync.md).

## Responsibilities

| Boundary        | Owns                                                                                                                               |
| --------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `baza`          | Documents, schema validation, staging and commit, history, merge, search, backup/restore, and storage-key lifecycle.               |
| `baza-storage`  | Encrypted streams and container/index mechanics.                                                                                   |
| `baza-common`   | Shared low-level types, crypto helpers, locks, and filesystem publication primitives.                                              |
| `arhiv`         | Application schema, platform key-cache integration, local-file import/download workflows, assets, and HTTPS server/UI integration. |
| `arhiv-cli`     | CLI commands; currently calls both `arhiv` and `baza`.                                                                             |
| `arhiv-desktop` | Electron launch, server-child lifetime, certificate trust, and browser session setup.                                              |
| `arhiv-android` | Java WebView, JNI runtime/server lifetime, Android credential cache, and platform permissions.                                     |

Core dependency direction is `arhiv -> baza -> baza-storage -> baza-common`,
with higher layers also using shared helpers directly. The React UI communicates
through the [UI API](specs/api.md), rather than accessing storage.

## Runtime paths

```text
CLI commands ------------------------------> Arhiv / Baza
Electron -- child-process startup --+
Android --- JNI startup ------------+------> local Rust HTTPS server
Browser --- one-time bootstrap -----+                 |
                                                     v
                                                React UI
                                                     |
                                                typed UI API
                                                     |
                                                Arhiv / Baza
                                                     |
                                           encrypted files and blobs
```

Electron receives startup material through the stderr marker protocol; Android
receives it through JNI. Both pin the supplied certificate and establish the
session cookie directly. Browser launch uses a separate one-time bootstrap token.
The server binds IPv4 loopback. See the
[runtime protocol](specs/runtime-protocol.md) and
[authentication contract](specs/authentication.md).

Unlock supplies the storage key. Database open holds storage ownership while
merging recognized storage files, performing a supported data migration, and
loading or rebuilding local state. Refresh imports incoming snapshots only
when no staged changes exist.

Edits are staged before commit. Commit makes the entire staging set durable
atomically at the domain boundary; concurrent revisions remain conflicted until
their staged merge is committed. See the [domain model](domain-model.md) and
[merge contract](specs/merge-conflicts.md).

## Data and trust boundaries

- **Committed storage:** the encrypted database holds snapshots; encrypted blobs
  hold asset content separately. Document types are compiled into the application.
- **Local state:** staging, document locks, and the search index live separately
  from synchronized committed storage. Backups exclude them.
- **Credentials:** the password wraps the storage master key; each asset has its
  own blob key. Platform caches are unlock conveniences, not recovery copies.
- **Synchronization:** external file-sync tools or manual transfer deliver
  storage files. Merge treats received snapshots as trusted committed history;
  it does not repeat local staging validation.
- **Recovery:** backups bind copied encrypted artifacts with an authenticated
  manifest, but capture is not an all-files transactional snapshot. Filesystem
  transactions provide in-process rollback, not crash-safe multi-file atomicity.

The [threat model](system-threat-model.md) owns security goals and residual
risks. The [storage format](specs/storage-format.md),
[key lifecycle](specs/key-lifecycle.md), and
[backup/restore contract](specs/backup-restore.md) own detailed guarantees.

## Development and operations

Debug UI serving reads filesystem assets; release serving embeds them.
Development and production state are separated by the production-mode feature
and platform configuration. Desktop packaging is established for Arch Linux;
Android uses a WebView and JNI inside its app process.

Use [Contributing](../CONTRIBUTING.md) for builds and checks,
[owner workflows](guides/user-workflows.md) for product outcomes, and the
[storage upgrade runbook](runbooks/storage-upgrade.md) for upgrade/recovery.
[Architecture decisions](adr/README.md) preserve significant
choices and their rationale.