# Personal Device Management

Status: draft; not implemented.

One owner manages access to Arhiv from their own devices. This proposal covers
device membership and authorization. [P2P sync](p2p-sync.md) covers connections
and the exchange of committed changes.

The scope is one owner's devices; hosted accounts, multi-user sharing, roles,
and permanent identity continuity are out of scope. Existing
[UI authentication](../specs/authentication.md) governs local access only.

## Vocabulary

| Term                    | Meaning                                                                                  |
| ----------------------- | ---------------------------------------------------------------------------------------- |
| **Arhiv**               | The owner's records, files, and committed history. Each device keeps a local copy.       |
| **Device group**        | The set of devices authorized to sync an Arhiv.                                          |
| **Pair device**         | Add a new or previously removed device to the device group.                              |
| **Pairing code**        | A code a device shows so the other device can find and verify it during **Pair device**. |
| **Remove device**       | Revoke a device's permission to sync directly.                                           |
| **Restore from backup** | Recover Arhiv data from a backup. Emergency restore starts a fresh device group.         |
| **Arhiv password**      | The password used to unlock the local copy and confirm sensitive actions.                |

## Agreed behavior

Devices have equal authority. Any authorized device can approve pairing or
removal without a third device. Authority belongs to devices in the group;
there is no shared owner authority or offline recovery signing authority.

Verify the Arhiv password locally at the point where pairing or removal takes
effect. An unlocked UI, cached storage key, or OS authentication is not a
substitute. This guards against misuse of an unattended unlocked UI, not a
compromised OS or extracted secrets.

### Pair device

The proposed first-run choices are **Create new Arhiv**, **Pair device**, and
**Restore from backup**. The proposed **Devices** settings list the owner’s
devices and offer **Pair device** and **Remove device**.

Open **Pair device** on both devices. The device being added displays the
pairing code as a QR code and as copyable text. On the approving device, scan,
paste, or type the same code, then confirm with the Arhiv password. The text
options work without a camera, and the code may be as long as needed.

The code finds and verifies the intended device and binds approval to the
pairing attempt. Each new device generates its own private identity. Provide
data keys only after approval, over the authenticated, protected pairing
connection.

### Remove device

Confirm with the local Arhiv password. A failed or cancelled confirmation leaves
membership unchanged. Each device stops syncing directly with the removed device
once it learns about the removal. An offline device may still use its older view
of the group until it receives the update. Removal cannot erase existing copies
or invalidate extracted data keys. Coordinated data-key rotation is deferred;
if introduced, it is a separate operation the owner explicitly requests.

Removal wins over concurrent approvals made by the removed device. For example,
if the laptop removes the phone while the phone is pairing a tablet, the
tablet needs fresh approval from a still-authorized device. Any data or keys
the phone already gave the tablet remain there.

### Pair a previously removed device

Use **Pair device** even if the only two devices in the group removed each
other. Sync between them pauses until the owner confirms once with the Arhiv
password. Both devices keep their local copies and stay in the same group.
Other devices can sync with them as soon as they learn about the approval.

An ordinary network reconnection or an old pairing approval must not undo
removal. If a removal was not known when the owner approved **Pair device**,
the removal wins once it is learned. The owner can then use **Pair device** again.

### Change the Arhiv password

The [key lifecycle](../specs/key-lifecycle.md) separates the password wrapper
from the storage master key. A password change rewraps the key; it does not
revoke existing copies.

In alpha, change the Arhiv password from any authorized device. On each other
device, unlock with its previous password and enter the new password once.
If the owner no longer knows the previous password, they can pair the device
from a current peer, but this does not recover local-only changes from the
locked copy.

If the owner changes the password independently on disconnected devices, each
keeps its local password until the owner sets a new shared password.

### Restore from backup without a surviving device

Restore the encrypted backup with the recovery credentials and start a fresh
device group with a fresh device identity. Recovery credentials grant data
access, not authority over the old device group. Prefer the existing
password-protected key export and backup capabilities over a separate key
hierarchy. **Recovery kit** is a possible name, not a new format.

The old group stays separate: restore neither remotely disables it nor
reactivates its identities or membership. If an old device turns up, use
**Pair device** to add it to the new group. Its missing committed changes
[merge automatically after pairing](p2p-sync.md#transfer-and-merge).
Keep emergency restore into a fresh group distinct from ordinary in-place
backup restore.

## Design and validation work

- **Membership.** Define how devices verify and exchange decisions and apply
  the agreed rules regardless of arrival order. A candidate is to link signed
  membership decisions to the earlier decisions they were based on. Whether
  to use the Iroh identity or a separate signing key is open, as are the wire
  format and membership merge algorithm. Known-old membership must not regain
  authority through normal sync; the design does not prove that an isolated
  device has the globally newest membership.
- **Pairing.** Define what the pairing code contains, how approval is bound
  to the intended device, and what happens if pairing is interrupted. Verify
  all three code entry methods and that missing, wrong-password, or cancelled
  confirmation leaves authorization unchanged and discloses no data keys.
- **Removal.** Verify that a device enforces a removal once it learns about it,
  including concurrent approvals by the removed device and removals not known
  at pairing time. Verify that mutual removal pauses sync and that one fresh
  **Pair device** approval restores it without losing local data. Describe
  propagation delay and retained copies and keys in the UI.
- **Arhiv password.** Define local password wrapping, how the owner resolves
  conflicting password changes, confirmation lifetime, and failed-attempt
  handling. Recovery-key export is a candidate additional critical action.
- **Restore.** Define how backups handle device secrets and membership. Verify
  emergency restore without surviving devices starts a fresh group, with no
  hidden signing authority, and hands returning devices to sync after pairing.

A designated management device could simplify ordering, but conflicts with
equal authority to pair or remove devices. A shared owner private key or
recovery signing authority adds privileged authority that fresh-group recovery
does not need. Revisit identity continuity only if a concrete owner workflow
needs it after total device loss.

When delivered, reconcile the affected
[key lifecycle](../specs/key-lifecycle.md),
[system threat model](../system-threat-model.md),
[backup and restore](../specs/backup-restore.md), and
[owner workflows](../guides/user-workflows.md) contracts. These still describe
current behavior.

## References

- [`@localfirst/auth`](https://github.com/local-first-web/auth) — decentralized
  authentication and authorization backed by signed history.
- [p2panda: Access Control in Decentralised Systems](https://p2panda.org/2025/07/28/access-control.html)
  — how removal invalidates concurrent approvals and the decisions that depend
  on them, including an example of mutual removal.
- [Keybase: Server Security](https://book.keybase.io/docs/server) — per-device
  keys and signed device changes. Its public Bitcoin checkpoint is not part
  of this proposal.