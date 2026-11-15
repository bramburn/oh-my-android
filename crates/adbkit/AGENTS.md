# AGENTS.md — crates/adbkit (scope: crates/adbkit/)

All adb access for the Windows app. Read the root `AGENTS.md` first.

## Absolute rules

1. **This crate is the only place that spawns a process.** Root `AGENTS.md` invariant #1; CI enforces it with
   a `Select-String 'Command::new'` guard over `src-tauri/` and `src/`. If you need a process elsewhere, the
   design is wrong.
2. **No `tauri` dependency, ever.** `cargo tree -p adbkit` must not contain `tauri`/`wry`/`tao`. That is what
   makes every module testable with no UI and no window.
3. **No device, no SDK, and no real adb in tests.** `run`/`run_server_cmd` are tested against `cmd.exe`;
   parsers are tested against fixtures. A test that needs a phone is an integration test behind
   `ADB_E2E=1` + an explicit serial (`docs/windows-port/17-testing-strategy.md` §7).
4. **argv, never a shell string.** Arguments are passed to `Command::new(adb).args(...)`. Only text that
   must reach the *device* shell is quoted, and only through a single helper (not yet needed in MVP).
5. **Every spawn** sets `CREATE_NO_WINDOW`, `stdin(null)`, `kill_on_drop(true)`, is assigned to the
   process-wide job, and has a timeout (20 s MVP, upstream 60 s — `Sources/Core/Shell/ShellRunner.swift:33`).
6. **`start-server`/`kill-server` must use `run_server_cmd`** (all three stdio null). A piped client can let
   the daemon inherit the pipe; upstream documents that hazard at
   `Sources/Core/Shell/ShellRunner.swift:58-67` and `:123-131`. Note: the hang did **not** reproduce with adb
   37.0.1 on Windows (`docs/windows-port/21-mvp-log.md` §3) — the rule is kept regardless because it costs
   nothing and matches upstream.
7. **Windows text rules**: trim `\r` before parsing/compare; decode with `String::from_utf8_lossy`; never
   `unwrap()` a parse.
8. **No `unwrap()`/`expect()` outside `#[cfg(test)]` and `main`.** Errors are `Result<T, AdbError>`.

## Layout

| File | Owns |
|---|---|
| `src/lib.rs` | module list + `AdbError` (the error taxonomy) |
| `src/parser.rs` | pure parsers (`devices -l` today). Add one module per adb output shape |
| `src/runner.rs` | the only `Command::new`. Two shapes: `run` (output) and `run_server_cmd` (null stdio) |
| `src/sdk.rs` | locate `adb.exe`; `locate_with` takes injected env so tests never touch the machine |
| `src/winjob.rs` | the process-wide Job Object (`KILL_ON_JOB_CLOSE`) |
| `tests/fixtures/` | verbatim adb output; `-text` in `.gitattributes` so CRLF survives |
| `tests/parser.rs` | golden tests, one per fixture + edge cases |

## Adding a parser

1. Capture the output verbatim into `tests/fixtures/<command>_<case>.txt` (never hand-write it), redact
   serials and models.
2. If it carries CRLF or invalid UTF-8, confirm the fixture still has it after `git add`
   (`git show :path | file -`).
3. Write `parse_x` in `src/parser.rs`, returning `Result<_, AdbError>` when partial parsing is not safe.
4. Add the golden test first, watch it fail, then implement.
5. Update `docs/architecture/06-adb-integration.md` if the command is new.

## Error taxonomy

`AdbError::{Spawn, Io, Timeout, NonZero{stderr}, SdkMissing}` → mapped to `AppError { code, message }` in
`src-tauri/src/error.rs`. Codes are stable: `ADB_SPAWN_FAILED`, `ADB_IO_ERROR`, `ADB_TIMEOUT`, `ADB_FAILED`,
`SDK_NOT_FOUND`. Do not leak absolute host paths into `message` (test:
`spawn_errors_do_not_leak_paths_to_the_ui`).

## Known measured behaviour (keep the comments honest)

The Job Object takes the adb **daemon** down with the app, because job membership is inherited. That
contradicts upstream (daemon survives) and is a deliberate MVP trade-off — see the module docs in
`src/winjob.rs` and `docs/windows-port/21-mvp-log.md` §3 D5. When long-lived streams land, the
daemon-spawning command must break away from the job.