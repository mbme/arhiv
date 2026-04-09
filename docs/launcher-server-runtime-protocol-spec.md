# Arhiv Launcher-Server Runtime Protocol Spec

Status: implementation-aligned (current behavior)

Scope: startup/shutdown contract between launchers (CLI/Desktop/Android) and the Arhiv server process, including discovery data, lockfile semantics, and single-instance behavior.

## 1. Runtime Surfaces

- CLI server command: `arhiv server [--json] [--port <u16>]`.
- Desktop launcher: spawns CLI server process and parses server info marker from stderr.
- Android launcher: invokes server start/stop via JNI and receives a `ServerInfo` object.

## 2. ServerInfo Payload Contract

Canonical Rust struct (`serde` camelCase):

- `uiUrl: string`
- `uiUrlWithAuthToken: string`
- `healthUrl: string`
- `certificate: number[]` (DER bytes)
- `authToken: string`

URL construction rules:
- `uiUrl = https://localhost:<port>/ui`
- `uiUrlWithAuthToken = <uiUrl>?AuthToken=<authToken>`
- `healthUrl = https://localhost:<port>/health`

Notes:
- Desktop TypeScript `ServerInfo` currently reads a subset (`uiUrl`, `healthUrl`, `certificate`, `authToken`) and ignores extra fields.
- Android Java `ServerInfo` currently maps `uiUrl`, `authToken`, `certificate`.

## 3. CLI JSON Marker Protocol (Desktop)

When `--json` is passed:

- Server writes exactly one line to stderr with prefix:
  - `@@SERVER_INFO: <json>`
- `<json>` is serialized `ServerInfo` object in camelCase.

Desktop parser behavior:
- reads stderr line stream,
- takes first line with `@@SERVER_INFO:` prefix,
- parses JSON payload,
- computes certificate fingerprint from `certificate` bytes.

Failure modes:
- no marker line => desktop startup fails (`No server info marker found`).
- invalid JSON after marker => desktop startup fails.

## 4. Port and Lockfile Semantics

Server lock path:
- `<state_dir>/arhiv-server.lock`

On server startup (`ArhivServer::start`):
1. Acquire exclusive lock (`LockFile::must_lock`) on lockfile.
2. Write requested port into lockfile.
3. Start HTTPS server.
4. Resolve actual bound port from server handle.
5. Overwrite lockfile with actual port.

Lockfile content:
- ASCII decimal `u16` port value.

Port discovery helper (`ServerInfo::get_server_port`):
- tries to acquire same lockfile.
- if acquire succeeds => server considered not running (`None`).
- if acquire fails => reads lockfile port and returns `Some(port)` if non-zero.

Lock lifecycle:
- server keeps lock for process lifetime.
- on drop, lock is released and lockfile is removed (`LockFile` cleanup for `must_lock`).

## 5. Single-Instance Semantics

Server single-instance:
- lockfile exclusive lock enforces one server instance per `state_dir`.
- second server start for same `state_dir` fails lock acquisition.

Desktop app single-instance (UI process):
- Electron uses `app.requestSingleInstanceLock`.
- second desktop instance exits and signals action to first instance.

Android process model:
- JNI layer uses global mutex-protected singleton handles for runtime and server.
- starting when runtime/server already exists fails (`Runtime already started` / `Server already started`).

## 6. Shutdown Semantics

CLI path:
- server runs until shutdown signal, then `server.shutdown()` is awaited.

Desktop path:
- desktop process owns spawned CLI child process and kills it on process exit.

Android path:
- `ArhivServer.stopServer()` triggers JNI stop,
- runtime awaits `server.shutdown()` then performs runtime shutdown timeout.

## 7. Health Endpoint Contract

- Endpoint: `GET /health`
- Expected status: `200 OK`
- Response includes no-cache headers.

## 8. Known Limitations

- CLI marker protocol is stderr text-line based, not framed IPC.
- Lockfile stores only port; no pid or richer metadata.
- Protocol versioning field is not present in `ServerInfo` payload.

## 9. Source of Truth (Code References)

- `arhiv-cli/src/bin/arhiv.rs`
- `arhiv/src/server/mod.rs`
- `arhiv/src/server/server_info.rs`
- `arhiv/src/server/server_lock.rs`
- `baza-common/src/lock_file.rs`
- `arhiv/src/support/http_server.rs`
- `arhiv-desktop/src/arhiv.ts`
- `arhiv-desktop/src/index.ts`
- `arhiv-android/src/lib.rs`
- `arhiv-android/app/src/main/java/me/mbsoftware/arhiv/ArhivServer.java`
- `arhiv-android/app/src/main/java/me/mbsoftware/arhiv/ServerInfo.java`
