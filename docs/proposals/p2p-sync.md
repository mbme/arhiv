# P2P Sync and Device Trust

Status: draft; not implemented.

This proposal designs synchronization between one owner's devices, starting with
pairing, removal, and recovery rather than a general account system. It separates
agreed product constraints from candidate mechanisms and unresolved security
rules. Approval of the direction does not imply that the protocol is complete.

## Current behavior and motivation

Arhiv is a single-owner, local-first database. Its
[domain model](../domain-model.md) has no account object. Existing
[UI authentication](../specs/authentication.md) protects local access;
it does not authorize remote synchronization.

The [key lifecycle](../specs/key-lifecycle.md) separates a password
wrapper from the storage master key. Password changes rewrap that key rather than
revoking existing copies. Asset decryption keys are stored in database metadata.
The [merge contract](../specs/merge-conflicts.md) already handles synchronized
snapshots and concurrent revisions, but treats received snapshots as trusted
committed history.

The desired experience is "my data and my devices": introduce another device,
let connected devices synchronize, and remove devices without introducing a
privileged everyday device or a permanent account identity.

## Agreed design constraints

- Devices have equal authority. Any authorized device can enroll or revoke
  another without a particular third device being online.
- Synchronization requires both participating devices to be running and online.
  There is no offline mailbox or store-and-forward service in this scope.
- Use Iroh for P2P connectivity. Arhiv owns membership and authorization.
- Conflicting membership decisions pause synchronization across the disputed
  relationship and require explicit human reconnection. Arhiv does not silently
  select a winner. The exact conflict and reconciliation rules remain open.
- Critical actions require local reauthentication with the Arhiv password,
  including device removal. An unlocked UI, cached storage key, or OS
  authentication does not substitute for this confirmation.
- Device removal stops synchronization once each peer receives the revocation.
  It neither erases existing copies nor invalidates extracted data keys.
- Coordinated data-key rotation is deferred. If introduced, it is a separate
  operation triggered by an explicit human request, not an automatic consequence
  of removal.
- Emergency recovery restores data into a fresh device group. Surviving devices
  are paired again; preserving the previous trust identity is not required.
- No designated management device, shared owner authority, or special offline
  recovery signing authority is introduced in this design.

The initial scope is one person's devices. Multi-user sharing, roles, a hosted
account service, and permanent identity continuity are outside it.

## Owner experience

The screens and wording below are proposals, not established UI contracts.

```text
Settings
  Devices
    This laptop
    Phone              [Remove]
    [Add device]

  Backup & recovery
    [Create recovery kit]
    [Restore backup]
```

### Connect a new device

First-run setup offers three distinct paths:

```text
Welcome to Arhiv
  [Create new Arhiv]
  [Connect to existing Arhiv]
  [Restore from backup]
```

Restore is a recovery path, not part of Add device.

```text
Existing device                        New device
[Add device]                           [Connect to existing Arhiv]
      |                                         |
      +--- QR code / private invitation ------->+
      |                                         |
      +<-------- Pairing request ---------------+
      |
Verify intended device
Confirm with Arhiv password             (proposed critical action)
      |
      +--- Authorize and provision ------------>+
                                                |
                                         Initial sync
```

An invitation introduces a reachable endpoint; possession of connection
information alone does not authorize database access. The candidate invitation
contains endpoint identity, connection hints, and a short-lived one-use pairing
secret. Exact encoding, expiry, identity verification, and approval binding are
unresolved.

The new device generates its own private identity rather than cloning an existing
device's identity. Data keys are provisioned only after successful authorization
over the authenticated, protected pairing connection. A candidate UX lets each
device choose its own local Arhiv password; password wrapping and backup
consequences need design before this becomes a requirement.

### Find and reconnect to paired devices

```text
Invitation or remembered peer identity
                  |
           Iroh connectivity
        (direct or relay-assisted)
                  |
     Arhiv authorization and dispute check
                  |
           Authorized data sync
```

Arhiv remembers paired device identities rather than depending on fixed IP
addresses. The implementation must verify the selected Iroh version's addressing,
lookup, relay, and privacy behavior before choosing configuration or a wire
format. Connection relays are not database backups or offline delivery services.

Proposed owner-visible states distinguish offline, connecting, syncing,
up-to-date, authorization conflict, and failure. "Up-to-date" must describe a
completed exchange with a particular peer, not imply that all offline devices
have the same data.

### Remove a device

Proposed confirmation:

> Remove Phone? Your other devices will stop syncing with this phone once they
> receive this change. This cannot erase data already on the phone or revoke
> data keys it already holds.

The owner confirms with the local Arhiv password. A failed or cancelled
confirmation leaves membership unchanged.

```text
Laptop removes Phone
         |
         +--- Laptop refuses further sync with Phone
         |
         +--- Desktop receives revocation later
                        |
                        +--- Desktop refuses sync with Phone
```

Removal is not instant global knowledge. An offline device can continue operating
under membership information it has not yet learned is obsolete. The protocol
must define how membership updates are processed before data exchange and how
removal affects existing connections.

### Resolve a membership dispute

Example: laptop and phone each remove the other while disconnected. Once the
conflicting decisions become known, affected synchronization pauses. The UI
explains that device authorization needs attention and offers explicit
reconnection rather than an automatic winner.

This is an agreed outcome, not a complete authorization algorithm. In particular,
reconnection must define which decisions it supersedes, which devices it
reauthorizes, and how other peers learn that resolution. Ordinary reconnection or
an old invitation must not silently undo removal. Resolution preserves local data
rather than discarding edits to repair trust.

### Recover without a surviving device

```text
Encrypted data backup + recovery credentials
                      |
                Restore data
                      |
       Fresh device identity and device group
                      |
          Pair surviving devices explicitly
```

Recovery credentials provide data access; they do not sign an override of the old
device group. Prefer reusing the existing password-protected key export and
backup capabilities over inventing another key hierarchy. "Recovery kit" is a
candidate product name, not a new format decision.

The old group is not remotely disabled by restore. Rejoining devices require an
explicit transition, including a way to preserve and reconcile unsynchronized
changes. Restored backups must not silently reactivate old device identities or
membership. Ordinary in-place backup restore and emergency fresh-group recovery
must remain distinguishable workflows.

## Trust, secrets, and ownership

Candidate responsibility split:

| Material or decision               | Responsibility                                                                 |
| ---------------------------------- | ------------------------------------------------------------------------------ |
| Device private identity            | Local installation; unique to that installation                                |
| Membership and revocation evidence | Arhiv verifies and propagates authorization decisions                          |
| Data decryption keys               | Authorized data-holding devices; existing encrypted backup/recovery mechanisms |
| Arhiv password                     | Local unlock and explicit critical-action confirmation                         |
| Endpoint routing and transport     | Iroh connectivity, separate from Arhiv authorization                           |

Signed membership decisions are a candidate mechanism for verifiable propagation.
Whether the Iroh identity also signs these decisions or a separate device signing
key is justified remains unresolved. There is no settled wire schema or
membership merge algorithm.

The network boundary belongs in `arhiv`; completed local data, persistence, and
merge behavior remain owned by `baza`. Remote peers do not become clients of the
loopback UI API. Existing browser and launcher authentication remains separate.

Password confirmation protects against use of an unattended unlocked UI, not a
compromised OS or extracted secrets. It must be enforced at the action boundary,
not only by a UI dialog. Enrollment approval and recovery-key export are proposed
additional critical actions; the full action list and confirmation lifetime are
open decisions.

## Security and protocol decisions still needed

1. **Membership conflict semantics.** Define causal ordering, propagation,
   replay/rollback handling, conflict scope, and explicit resolution. Consider a
   phone enrolling a tablet concurrently with the laptop revoking that phone.
   The tablet's authorization cannot be decided merely by checking a signature
   or comparing wall-clock timestamps.
2. **History trust.** Decide whether to trust the authorized peer delivering
   snapshots or require original-author provenance. The current merge boundary
   does not establish per-author authorization. Removing a connection does not
   by itself reject that device's changes forwarded by another peer.
3. **Pairing security.** Specify one-use invitations, expiration, intended-peer
   verification, password confirmation, replay resistance, and interruption
   behavior. An invitation recipient must not gain access before approval.
4. **Local password policy.** Specify device-local wrapping, exact critical
   actions, per-action versus short-lived confirmation, failed-attempt handling,
   and behavior when the owner has a valid key cache but has forgotten the
   password. Cached unlock must not bypass required password confirmation.
5. **Sync lifecycle.** Decide whether sync runs while locked, what data and
   membership state it may access, and how received data waits while changes are
   staged. Preserve the existing staging gate unless explicitly redesigned.
6. **Safe transfer and recovery.** Define the committed-data transfer unit, blob
   handling, resource limits, interruption recovery, compatibility checks, and
   publication boundary. Specify handling of device secrets and membership in
   backups and preservation of surviving devices' unsynced data after recovery.
7. **Connectivity operations.** Select relay/lookup configuration and document
   exposed metadata, service dependencies, and actionable offline/failure states.

These are implementation blockers, not reasons to reopen the agreed product
constraints. Known-old membership must not regain authority through normal sync;
proving that an isolated device has the globally newest membership is a different
problem that this proposal does not claim to solve.

## Alternatives and tradeoffs

- **Keep file-based synchronization only:** avoids a new remote trust boundary,
  but does not deliver integrated device pairing and P2P synchronization.
- **Designated management device:** simplifies authority ordering, but conflicts
  with equal-authority enrollment and removal. Rejected for this scope.
- **Shared owner private key or recovery signing authority:** adds privileged
  authority and recovery semantics that current workflows do not require.
  Recovering data into a new group is the chosen simpler direction.
- **Automatic key rotation on removal:** would couple membership changes to
  storage-key distribution and recovery. Deferred; future rotation must be an
  explicit owner-requested operation.
- **Offline store-and-forward:** would require persistent intermediary storage
  and additional authorization/lifecycle rules. Excluded by the online-only
  availability contract.

Fresh-group recovery deliberately requires re-pairing and does not retire the
old group remotely. Revisit identity continuity only if a concrete owner workflow
requires preserving relationships across total device loss.

## Acceptance scenarios and delivery boundary

These are proposed validation scenarios, not checks already implemented or run:

- Two online devices pair through a deliberate approval; another device need not
  be online. The resulting devices have equal enrollment/removal authority.
- Missing approval, wrong-password confirmation, and cancelled confirmation leave
  authorization unchanged and disclose no data keys.
- Expired and consumed invitations cannot authorize another connection.
- A peer that has learned a revocation refuses data sync with the removed device;
  the UI accurately describes propagation delay and retained data/key access.
- Conflicting membership decisions pause affected sync without deleting local
  data. Explicit resolution propagates without old decisions silently undoing it.
- An offline peer causes a waiting/offline state, not a claim that data has been
  delivered. Interrupted transfers do not publish partial committed storage.
- Received data respects existing staged-change and committed-history boundaries.
- Recovery works without surviving devices and creates a fresh device group;
  reconnecting surviving devices preserves unsynchronized data.
- Removal performs no coordinated key rotation. Restore introduces no hidden
  recovery signing authority.

Before implementation, resolve the security rules and transfer boundary, then
identify focused tests for them. Any persistent format or document-contract
changes must follow the existing migration/versioning rules; this draft does not
select a migration or claim storage compatibility.

Existing living specs continue to describe current behavior. When this design is
accepted and delivered, reconcile the narrow affected contracts, particularly
[key lifecycle](../specs/key-lifecycle.md),
[system threat model](../system-threat-model.md),
[merge behavior](../specs/merge-conflicts.md),
[backup and restore](../specs/backup-restore.md), and
[owner workflows](../guides/user-workflows.md). Keep proposal rationale distinct from
implemented guarantees.