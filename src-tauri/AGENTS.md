# AGENTS.md — src-tauri (scope: src-tauri/)

Tauri shell. Thin. Everything that talks to a device lives in `crates/adbkit`. Read the root `AGENTS.md`
first.

## Rules

1. **Thin commands only.** A `#[tauri::command]` resolves state, calls `adbkit`, maps the error. No parsing,
   no process spawning, no business logic, no loops.
2. **No `Command::new` here.** CI fails the build if it appears outside `crates/adbkit` (root invariant #1).
3. **Tauri v2 only.** `tauri::api::*`, `allowlist`, or a `tauri.conf.json` with a top-level `tauri` key are
   bugs. Config is validated against `https://schema.tauri.app/config/2`.
4. **No new plugin and no new dependency without an ADR.** MVP has none, deliberately: no `shell`, no `fs`,
   no `store`, no `dialog`. Native dialogs come later with a capability scope decision.
5. **Capabilities are least privilege.** `capabilities/default.json` grants `core:default` to `main` only.
   Remember: capabilities gate *plugin* commands; app commands must validate their own preconditions in Rust.
6. **Errors cross the boundary as `AppError { code, message }`** (`src/error.rs`). `message` is a sentence for
   a human; never include a stack trace, an absolute path, or a device serial's full value.
7. **Never panic on device data.** A bad parse is `ADB_FAILED`, not a crash. `expect` is allowed only for
   "the app cannot start at all" (`run()`).
8. **`dist/` is generated.** `tauri::generate_context!` embeds `frontendDist` at compile time, so
   `npm run build` must run before `cargo build` in CI. Never commit `dist/`.
9. **Startup must not block on adb.** `AppState::new` resolves the SDK path synchronously (a few `is_file`
   checks); all adb calls are `async` commands.

## Files

| File | Owns |
|---|---|
| `src/lib.rs` | builder wiring, `invoke_handler`, managed state |
| `src/main.rs` | bin entry, `windows_subsystem = "windows"` in release |
| `src/state.rs` | `AppState` — the cached `adb.exe` path (re-located when empty) |
| `src/commands.rs` | `list_devices`, `adb_start`, `adb_stop` |
| `src/error.rs` | `AppError` + the `adbkit::AdbError` mapping |
| `tauri.conf.json` | window, CSP, bundle, NSIS per-user |
| `capabilities/default.json` | the entire permission grant |
| `icons/` | generated monochrome placeholders — never upstream assets (`docs/legal.md` §1) |

## Adding a command

1. Add the function to `commands.rs`, register it in `generate_handler!`, type its arguments and return.
2. Mirror the types in `src/lib/ipc.ts` (the only module that calls `invoke`).
3. Map every new failure to a stable `code` in `error.rs` and handle it in the UI — an unhandled code must
   still render a sentence.
4. If the command mutates the device, it is on the human-approval list (`AGENTS.md` §7): the UI must confirm
   and the docs must say so.
5. Add a row to `docs/windows-port/12-ipc-contract.md`.

## MVP deviations in force

- **O1**: no hotplug; the UI updates on Refresh only.
- **O2**: Start/Stop targets the **adb server**; `kill-server` is behind a confirmation dialog.
Both are recorded in `docs/windows-port/21-mvp-log.md` §2 and must be reverted deliberately, not drifted away
from.