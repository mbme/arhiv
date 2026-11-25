# P2P Sync

Status: draft; not implemented.

This proposal covers connections and data exchange between one owner's devices.
[Personal Device Management](personal-device-management.md) defines which
devices are authorized and how that authorization changes.

## When sync runs

**Sync** exchanges committed changes between authorized devices. It starts
automatically when both devices are running, online, and unlocked. Arhiv waits
for the local staging set to clear before applying incoming changes.

Arhiv uses Iroh for direct or relay-assisted connectivity. It remembers paired
device identities so they can reconnect as addresses change. Relays help online
devices connect; they provide neither offline delivery nor backups.

```text
Remembered device identity
            |
     Iroh connectivity
  (direct or relay-assisted)
            |
 Authenticate the peer and process
 membership updates before deciding
 whether to send committed changes
            |
    Authorized data sync
```

`arhiv` handles peer connections and authorization; `baza` handles local data,
persistence, and merge. Peer sync uses a separate protocol from the loopback UI
API, whose browser and launcher authentication remains local.

Proposed owner-visible states distinguish offline, connecting, syncing,
up-to-date, authorization conflict, and failure. “Up-to-date” describes a
completed exchange with a particular peer.

## History and alpha migration

In alpha, Arhiv trusts committed changes from an authenticated, authorized
sender, including changes it forwards from a removed device. Instance IDs
track revisions. Alpha authenticates the sender rather than independently
verifying original authorship with per-change signatures.

Existing Arhiv data moves through a manual or partly automated cutover. Bring
outstanding changes together before adopting a single baseline. Keep existing
history and its instance IDs. After cutover, the old file-sync copy is a
backup, not another live sync path for the same Arhiv.

## Transfer and merge

Start with the historical Changeset flow: request missing committed snapshots,
skip duplicates, and fetch any missing files before committing the received
changes. Adapt it to the current storage and merge code rather than reviving
the old network stack. Asset decryption keys are stored in database metadata.
The [existing merge rules](../specs/merge-conflicts.md) handle concurrent
revisions and treat received snapshots as trusted committed history.

When an old device is paired with a new device group after emergency
**Restore from backup**, automatically merge its missing committed changes.

## Design and validation work

- **Connection setup.** Verify the selected Iroh version's addressing, lookup,
  relay, and privacy behavior before choosing configuration or a wire format.
  Document exposed metadata, service dependencies, and actionable offline
  and failure states.
- **Authorization boundary.** Define how membership updates are processed
  before data exchange, how removals affect existing connections, and what
  membership state is accessible while locked. Test the agreed device
  management scenarios across peers.
- **Sync lifecycle.** Define how automatic sync stops on lock and how received
  changes wait while local changes are staged. Show a waiting or offline state
  for an offline peer.
- **Safe transfer and recovery.** Define the transfer format, file handling,
  compatibility checks, resource limits, and recovery from interrupted
  transfers. Verify that interrupted transfers don't publish partial committed
  storage, that authorized devices can forward changes without per-change
  signatures, and that returning devices' missing committed changes merge
  automatically after pairing.

File-based sync alone avoids a new remote trust boundary but doesn't provide
integrated pairing or P2P sync. Offline store-and-forward would add persistent
intermediary storage and authorization rules; it is outside the online-only
scope.

Before implementation, resolve the transfer boundary and identify focused tests.
Any persistent format or document-contract changes must follow the existing
[migration and versioning rules](../specs/migration.md). The draft sets the
cutover policy but does not specify a migration procedure or claim storage
compatibility. When delivered, reconcile the affected
merge behavior,
[system threat model](../system-threat-model.md), and
[owner workflows](../guides/user-workflows.md). These still describe current
behavior.

## References

The previous Arhiv Changeset implementation is in the parent of `687f8db3`
(`remove baza network sync`). See `baza/src/sync/changeset.rs`,
`baza/src/sync/connection_ext.rs`, and `baza/src/sync/sync_manager.rs`. The
tests in `baza/src/tests/sync.rs` cover conflicts, staged changes, and file
transfer.

- [Ink & Switch: Syncing Keyhive](https://www.inkandswitch.com/keyhive/notebook/05/)
  — how Beelay authenticates peers and syncs membership before deciding which
  documents to exchange.
- [Secure Scuttlebutt: Feeds and Messages](https://spec.scuttlebutt.nz/feed/messages.html)
  — an example of verifying original authorship independently of the sending
  peer. Arhiv does not adopt that approach for committed changes in alpha.