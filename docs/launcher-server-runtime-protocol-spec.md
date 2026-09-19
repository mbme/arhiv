# Arhiv Launcher-Server Runtime Protocol Spec

Scope: startup/shutdown contract between launchers (CLI/Desktop/Android) and the Arhiv server process, including discovery data, lockfile semantics, and single-instance behavior.

## 1. Runtime Surfaces

- CLI server command: `arhiv server [--json] [--browser] [--port <u16>]`.
- Desktop launcher: spawns CLI server process and parses server info marker from stderr.
- Android launcher: invokes server start/stop via JNI and receives a `ServerInfo` object.

## 2. ServerInfo Payload Contract

Canonical Rust struct (`serde` camelCase):

- `uiUrl: string`
- `browserUrl: string`
- `healthUrl: string`
- `certificate: number[]` (DER bytes)
- `authToken: string`

URL construction rules:

- `uiUrl = https://localhost:<port>/ui`
- `browserUrl = https://localhost:<port>/auth?token=<one-time-browser-bootstrap-token>`
- `healthUrl = https://localhost:<port>/health`

`browserUrl` is for `arhiv server --browser` only. It establishes an authenticated cookie session and redirects to the clean `uiUrl`. Desktop and Android launchers use `authToken` to set the cookie directly.

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
- parses the JSON payload and passes it to desktop startup.

Failure modes:

- no marker line => desktop startup fails (`No server info marker found`).
- invalid JSON after marker => desktop startup fails.

## 4. Port and Lockfile Semantics

Server lock path:

- `<state_dir>/arhiv-server.lock`

On server startup:

1. Acquire an exclusive lock on the lockfile.
2. Write requested port into lockfile.
3. Start HTTPS server.
4. Resolve actual bound port from server handle.
5. Overwrite lockfile with actual port.

Lockfile content:

- ASCII decimal `u16` port value.

Port discovery:

- tries to acquire same lockfile.
- if acquisition succeeds, the server is considered not running.
- if acquisition fails, the lockfile port is returned when it is non-zero.

Lock lifecycle:

- server keeps lock for process lifetime.
- on shutdown, the lock is released and the lockfile is removed.

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

- server runs until a shutdown signal, then shuts down gracefully.
- With `--browser`, CLI logs `browserUrl`, starts the browser without blocking server startup, and reaps the browser child in the background.
- On Unix, the browser starts in its own process group so terminal signal delivery to the CLI does not terminate the browser.

Desktop path:

- desktop process owns spawned CLI child process and kills it on process exit.

Android path:

- the Java launcher triggers shutdown through JNI,
- the runtime waits for server shutdown and then applies its shutdown timeout.

## 7. Health Endpoint Contract

- Endpoint: `GET /health`
- Expected status: `200 OK`
- Response includes no-cache headers.

## 8. Known Limitations

- CLI marker protocol is stderr text-line based, not framed IPC.
- Lockfile stores only port; no pid or richer metadata.
- Protocol versioning field is not present in `ServerInfo` payload.