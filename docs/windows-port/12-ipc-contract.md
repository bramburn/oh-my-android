# 12 — IPC contract (UI ↔ Rust)

Single source of truth for the boundary. Rust commands are thin: parse → call `adbkit`/service → map error.
No business logic in a `#[tauri::command]`.

## 1. Transport choice per payload class

| Payload | Mechanism | Why |
|---|---|---|
| Request/response, small JSON | `#[tauri::command]` | simple, typed |
| Push notifications, low rate (<60/s) | `app.emit("…", payload)` | one-way, no handle plumbing |
| Progress + logcat batches | `tauri::ipc::Channel<…>` passed as a command arg | command resolves once, stream continues; **ordering guaranteed** |
| Binary >1 KB (screenshot bytes, H.264) | **not IPC** — `Response`/`Channel<InvokeResponseBody>` for one-shot ≤2 MB; loopback HTTP for continuous | see §4 |
| Frontend → Rust streaming (file upload) | repeated commands with an opaque `upload_id` | avoids request-body streaming complexity |

`VERIFIED` (crate 2.12.5, https://docs.rs/tauri/2.12.5/tauri/ipc/struct.Channel.html):

> "Any `serde::Serialize` value is sent as JSON. To stream binary data without the JSON overhead, send an
> `InvokeResponseBody::Raw` through a `Channel<InvokeResponseBody>` … or a `Response` … Note that a
> `Channel<Vec<u8>>` does not do this - `Vec<u8>` is `Serialize`, so it is sent as a JSON array."

⇒ **`Channel<Vec<u8>>` is a bug in the draft brief; never use it for binary.**

## 2. Command table

`S` = requires an explicit device serial. `mut` = mutates the device (confirm + ADB safety rules).
All errors are `AppError` (§3).

| Command | Args (TS) | Returns | S | mut | Rate | Notes |
|---|---|---|---|---|---|---|
| `app_info` | — | `{version, os, build, tauri}` | | | once | |
| `settings_get` | — | `Settings` | | | on mount | |
| `settings_set` | `{patch: Partial<Settings>}` | `Settings` | | | ≤5/s | emits `settings://changed` |
| `tools_locate` | `{manualPath?: string}` | `Toolchain \| null` | | | once | writes setting if manual |
| `tools_validate` | `{path: string}` | `{adbVersion, sdkRoot, emulator: bool}` | | | ≤1/s | runs `adb version` |
| `adb_restart_server` | — | `void` | | ✓ | ≤1/min | `kill-server` + `start-server` |
| `devices_list` | — | `Device[]` | | | on mount + event | snapshot |
| `device_set_active` | `{serial: string}` | `void` | | | ≤5/s | persisted |
| `device_info_get` | `{refresh?: bool}` | `DeviceInfo` | ✓ | | ≤1/2 s | **one batched shell** |
| `features_catalog` | — | `Feature[]` | | | once | mirrors `FeatureCatalog.swift:5-71` |
| `feature_run` | `{id, value?, confirm?}` | `void` | ✓ | ✓ | serialized | emits `action://state` |
| `screenshot_capture` | `{toClipboard: bool}` | `{path, width, height}` | ✓ | | ≤1/s | strips foldable warning prefix |
| `captures_open_folder` | — | `void` | | | ≤1/s | `opener` plugin |
| `apk_pick` | — | `string \| null` | | | ≤1/s | native dialog |
| `apk_install` | `{path, channel: Channel<Progress>}` | `void` | ✓ | ✓ | 1 concurrent | streamed progress |
| `app_foreground` | — | `{package, activity}` | ✓ | | ≤1/2 s | |
| `app_action` | `{action, package}` | `void` | ✓ | ✓ | serialized | force-stop/clear/uninstall/reset-perms/info |
| `ui_dump` | `{includeInvisible?}` | `UiNode` | ✓ | | ≤1/2 s | `uiautomator dump` + retry |
| `ui_audit` | — | `AuditReport` | ✓ | | ≤1/2 s | TalkBack order issues |
| `logcat_start` | `{serial, filters, channel}` | `{sessionId}` | ✓ | | 1 per device | |
| `logcat_stop` | `{sessionId}` | `void` | | | | |
| `logcat_export` | `{sessionId?, path?}` | `{path}` | | | ≤1/s | |
| `fs_list` | `{path}` | `FsEntry[]` | ✓ | | ≤2/s | |
| `fs_pull` | `{remote, local?, channel}` | `{path}` | ✓ | | 1 concurrent | progress |
| `fs_push` | `{local, remote, channel}` | `void` | ✓ | ✓ | 1 concurrent | progress |
| `fs_mkdir` / `fs_delete` | `{path}` | `void` | ✓ | ✓ | ≤2/s | delete = confirm |
| `wireless_pair` | `{addr, code}` | `void` | | ✓ | ≤1/10 s | `adb pair` |
| `wireless_connect` | `{addr}` | `void` | | ✓ | ≤1/5 s | `adb connect` |
| `mirror_start` | `{serial, channel}` | `{sessionId, port, token, width, height}` | ✓ | | 1 per device | **feature-gated**, ADR-0002 |
| `mirror_stop` | `{sessionId}` | `void` | | | | |
| `mirror_input` | `{sessionId, event}` | `void` | | ✓ | ≤120/s | throttled; not for MVP |

Commands **deliberately absent**: `shell_exec` (arbitrary shell is a security hole and bypasses the feature
catalog; a debug-only console in M4 is capability-gated and read-only by default), `file_read_any`
(`fs` plugin scope covers what the UI needs).

## 2.1 As built (MVP)

The table above is the **target** surface. What actually ships today is six commands; the names differ
(`setup_report` rather than `settings_get`/`tools_locate`) because the MVP has no settings screen — the guide
reads state instead of asking for it. Renaming is a UI+backend change and belongs to M2, not to drift.

| Command | Args (TS) | Returns | mut | Notes |
|---|---|---|---|---|
| `setup_report` | — | `SetupReport` | | read-only detection; the only process it runs is `adb version` |
| `choose_sdk_folder` | — | `SetupReport` | | native picker → validate → persist → re-detect. **Cancel is not an error** |
| `list_devices` | — | `Device[]` | | `adb devices -l`, parsed in `adbkit` |
| `adb_start` | — | `void` | ✓ (server) | `adb start-server`, all-null stdio |
| `adb_stop` | — | `void` | ✓ (server) | `adb kill-server`, behind `confirm()` (override O2) |
| `open_docs` | `{url: string}` | `void` | | allowlist checked **in Rust**; rejects anything outside four URLs |

`SetupReport` (`adbkit::setup`): `{studio, sdk: {root, adb, source} | null, adbVersion, emulator, avdCount,
envHome, customRoot, warning}`. The UI rule is exactly one branch: **`sdk === null` ⇒ the guide; otherwise the
device panel**, with the guide reachable from the header wrench button.

Error codes added by this milestone: `SDK_INVALID` (picked folder has no `platform-toolsdb.exe`),
`CONFIG_WRITE_FAILED` (choice works for the session only), `DIALOG_CLOSED`, `URL_NOT_ALLOWED`, `OPEN_FAILED`.
They follow the taxonomy below (`{code, message}` only — no `details`/`retryable`/`hint` yet; adding those
fields is a contract-wide change).

## 3. Error taxonomy

Rust: `thiserror` enums in `crates/adbkit` and services; mapped to `AppError` at the command boundary.

```rust
#[derive(Serialize)]
pub struct AppError {
    pub code: ErrorCode,      // machine-readable, stable
    pub message: String,      // user-facing sentence, already localized-ready
    pub details: Option<String>, // raw adb stderr, redacted
    pub retryable: bool,
    pub hint: Option<String>,    // "Reconnect the cable" / "Enable USB debugging"
}
```

| Code | Cause | UI behaviour |
|---|---|---|
| `ADB_NOT_FOUND` | no `platform-tools` | onboarding card → "Choose SDK folder" |
| `ADB_VERSION_UNSUPPORTED` | `adb version` < 1.0.40 | blocking banner |
| `ADB_SERVER_UNREACHABLE` | `:5037` refused | "Restart adb" button |
| `NO_DEVICE` | zero devices | panel state `NoDevices` |
| `DEVICE_NOT_FOUND` | serial gone between calls | refresh + toast |
| `MULTIPLE_DEVICES` | mutation without explicit serial | impossible by construction (`-s` mandatory) |
| `UNAUTHORIZED` | RSA prompt not accepted | state `Unauthorized` + instructions |
| `OFFLINE` | device offline/booting | state `Offline`/`Booting`, auto-retry |
| `DEVICE_DISCONNECTED` | mid-command | toast + refresh, `retryable: true` |
| `PERMISSION_DENIED` | e.g. `run-as` on non-debuggable | inline explanation |
| `PACKAGE_NOT_FOUND` | bad package | inline |
| `INSTALL_FAILED_*` | pass through `INSTALL_FAILED_…` from adb | verbatim, plus hint (`INSTALL_FAILED_UPDATE_INCOMPATIBLE` → "uninstall first") |
| `INSTALL_INVALID_APK` / `INSTALL_NO_SPACE` | mapped from adb stderr | verbatim |
| `TRANSFER_FAILED` | push/pull | partial-file cleanup |
| `PATH_INVALID` / `PATH_TRAVERSAL` | device-supplied name | refuse, log |
| `TIMEOUT` | watchdog (default 60 s, install 600 s) | cancel + toast |
| `CANCELLED` | user cancel | silent |
| `PARSE_ERROR` | output format changed | payload includes the offending line (redacted); user sees "Unsupported device output" |
| `MIRROR_*` | `MIRROR_START_FAILED`, `MIRROR_CODEC_UNSUPPORTED`, `MIRROR_DISCONNECTED`, `MIRROR_SECURE_DEVICE` | fallback ladder |
| `INTERNAL` | bug | "Something went wrong" + copyable details |

Any code added to this table is a UI change; keep it in sync with `src/lib/errors.ts`.

## 4. Bytes, not JSON

| Case | Mechanism | Status |
|---|---|---|
| Screenshot (1–3 MB PNG) once | write to disk in Rust, return the **path**; load in the UI via `convertFileSrc`/asset protocol | preferred |
| Screenshot to clipboard only | Rust writes clipboard directly (`arboard`/plugin), returns nothing | preferred |
| Small binary (<100 KB) | `tauri::ipc::Response` → `ArrayBuffer` | `VERIFIED` per §1 |
| Continuous video | loopback HTTP (`127.0.0.1:<ephemeral>`, per-session bearer token, `Origin` checked) + `ReadableStream` | see `13-mirroring-design.md` §4 |
| Never | `Channel<Vec<u8>>` | it JSON-encodes bytes |

**Why not a custom protocol for video:** `register_asynchronous_uri_scheme_protocol`'s responder takes
`http::Response<Vec<u8>>` — the whole body is materialised, so it is not a streaming response
(`VERIFIED`, https://docs.rs/tauri/2.12.5/tauri/struct.Builder.html). It also changes the page `Origin`
on Windows, which complicates the CSP. Loopback HTTP is a plain `TcpListener`/axum server we fully control.

## 5. Budgets

| Budget | Value | Enforcement |
|---|---|---|
| Max IPC messages/s per channel | **60** | coalescer in Rust; UI drops further if >4 frames behind |
| Max single IPC payload | **1 MiB** | debug assertion in the serializer wrapper; screenshots go to disk |
| Logcat batch window | **33 ms** or 200 lines | `streams/mod.rs` |
| Progress window | **100 ms** | same |
| Max events on `devices://changed` | 5/s (debounced 100 ms) | registry |
| Backpressure | logcat: drop-oldest (ring 50 k lines in UI); progress: last-wins; video: drop non-key frames | documented per stream |
| Command concurrency | 1 mutating action per device; N concurrent reads | `ActionRunner` semaphore |

## 6. Type sharing

| Option | Verdict |
|---|---|
| Hand-mirrored TS types in `src/lib/bindings.ts`, generated by a `cargo run --bin export-bindings` that serialises `schemars` JSON-Schema → TS | **chosen**: no extra runtime, no macro magic, fails CI on drift |
| `tauri-specta` | rejected for now: it is a third-party codegen that must track Tauri's IPC changes; revisit if hand-mirroring drifts twice |
| Runtime validation in the UI (zod) | **chosen** for `devices://changed` and `settings` only — cheap insurance against Rust-side bugs |

CI check: `npm run check:bindings` regenerates and diffs; a mismatch fails the build.

## 7. Event list

| Event | Payload | Rate |
|---|---|---|
| `devices://changed` | `{devices: Device[], active: string \| null}` | ≤5/s (debounced) |
| `device://state` | `{serial, state, detail?}` | on transition |
| `settings://changed` | `Settings` | on write |
| `action://state` | `{id, status: 'running'\|'ok'\|'error'\|'cancelled', error?}` | per action |
| `toast://show` | `{kind, message}` | ≤5/s |
| `tools://changed` | `Toolchain \| null` | rare |
| `logcat://dropped` | `{sessionId, dropped: number}` | ≤1/s |
| `mirror://stats` | `{sessionId, fps, kbps, drop, latencyMs?}` | 1/s (diagnostics) |