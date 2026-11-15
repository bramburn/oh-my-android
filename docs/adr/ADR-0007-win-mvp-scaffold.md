# ADR-0007 — Windows MVP scaffold (Tauri v2)

- Status: **Accepted** (the M0-scope prompt is the approval)
- Date: 2026-11-15
- Deciders: bramburn
- Supersedes: none. Amends ADR-0001 (framework) with an executable scaffold.

## Context

`Sources/**` is the reference implementation and nothing has been built yet. The MVP must be the
smallest runnable Windows app that proves the architecture: one adb choke point, a Job Object, a typed
IPC boundary, and a monochrome UI that shows the truth about device state. Upstream's entire device story
begins with `adb devices -l` (`Sources/Core/Android/ADBClient.swift:98-104`) and its recovery action is
`adb kill-server` / `start-server` (`:82-86`), so those two are the MVP.

## Decision

**We will scaffold a Tauri v2 app in a git worktree (`feat/win-mvp-tauri`) with a two-crate workspace and
exactly three IPC commands: `list_devices`, `adb_start`, `adb_stop`.**

### Workspace layout

```
Cargo.toml                 [workspace] members = ["crates/adbkit", "src-tauri"], resolver = "2"
rust-toolchain.toml        pinned stable MSVC
crates/adbkit/             parser + runner + sdk locator + job object (NO tauri dependency)
src-tauri/                 thin #[tauri::command] wrappers, capabilities, tauri.conf.json
src/                       Svelte 5 + TS + Vite, monochrome
```

### Product identity (working, provisional)

| Field | Value |
|---|---|
| `productName` | `Android Panel (working title)` |
| `identifier` | `dev.bramburn.androidpanel` |
| Window | label `main`, 420×640, native decorations, `withGlobalTauri: false` |
| Never | "Oh My Android", `se.royan.*`, upstream icons, upstream Sparkle feed/keys (`docs/legal.md` §1) |

### Overrides of `AGENTS.md` (recorded, not silently broken)

| # | Override | Rationale | Reverted in |
|---|---|---|---|
| **O1** | Device list updates on explicit **Refresh** only; invariant #4 (event-driven hotplug) deferred | smallest shippable step; `host:track-devices` needs its own socket client and tests | next milestone (`track-devices`) |
| **O2** | **Start/Stop acts on the adb server**, not emulators/devices. `kill-server` is on the human-approval list (`AGENTS.md` §7); the UI button *is* that approval and the confirm dialog is the enforcement. It is never called automatically and never from a test | it is upstream's own recovery action (`ADBClient.swift:82-86`); per-device emulator start/stop needs AVD enumeration + 180 s boot handling and is out of MVP | emulator control lands in a later milestone |

### Packaging

- `bundle.targets = ["nsis"]`, `nsis.installMode = "currentUser"` (no admin, no portable zip).
- `webviewInstallMode = downloadBootstrapper`.
- **Bring your own SDK**: nothing is bundled (`docs/legal.md` §2 option (a)); the app locates `adb.exe`
  and shows a `SDK_NOT_FOUND` state otherwise.
- No database, no persistence in the MVP. If state is ever persisted it is JSON with `"schemaVersion": 1`
  and any migration runs only in the main checkout, only with human approval
  (`docs/windows-port/21-mvp-log.md` §4).

### Dependencies (one line of justification each)

| Crate / package | Why |
|---|---|
| `tauri` 2.12.5 | the app shell, IPC, window, bundler |
| `tauri-build` 2.7.4 | build script for the same |
| `tokio` (process, time, macros, rt-multi-thread) | async process spawning + timeouts; `tauri::async_runtime` is tokio |
| `serde` / `serde_json` | command payloads, `Device` serialization |
| `thiserror` | `AdbError` taxonomy without hand-written `Display` |
| `windows-sys` 0.61.2 | `CreateJobObjectW` / `SetInformationJobObject` / `AssignProcessToJobObject` — no safe wrapper crate needed for three calls |
| `svelte` 5.57.3 | UI |
| `vite` 8.3.8 + `@sveltejs/vite-plugin-svelte` 7.3.1 | build/dev server |
| `typescript` 7.0.2 + `svelte-check` 4.7.8 | strict typing gate |
| `@tauri-apps/api`, `@tauri-apps/cli` 2.12.4 | IPC client + CLI, version-locked to the crate |

Explicitly **not** added in MVP: `tauri-plugin-store`, `tauri-plugin-shell`, `tauri-plugin-fs`,
`tauri-plugin-dialog`, `axum`, `rusqlite`, `scrcpy`. Each would be a new ADR.

## Consequences

- Positive: the three riskiest primitives (choke point, job object, typed IPC) are proven in ~2 days
  instead of being asserted in a plan; a real device list exists on day one.
- Negative: no hotplug, so the UI can be stale until Refresh; no persistence, so the SDK path is re-located
  on every command; native titlebar instead of the floating panel (deliberate, MVP).
- Neutral: `crates/adbkit` gains its first real tests, which become the fixture harness for every later
  parser (`docs/windows-port/17-testing-strategy.md` §3).

## Alternatives considered

| Option | Why not |
|---|---|
| Full M0 from `18-plan-and-backlog.md` (hotplug, 8 states, tray, fake-adb) | ~34 h vs ~10 h; defers the first runnable artifact |
| Scaffold with `npm create tauri-app` and grow it | template carries demo code, plugins and v1-era habits that must then be removed |
| Electron/.NET shell just for the MVP | contradicts ADR-0001 and would be throwaway |

## Evidence

- `Sources/Core/Android/ADBClient.swift:82-104` — the two adb commands this MVP exposes.
- `Sources/Core/Shell/ShellRunner.swift:58-67`, `:123-131` — why `start-server` must run with all stdio
  null (a piped client that implicitly spawns the daemon inherits the pipe and can hang).
- `docs/windows-port/10-version-pin.md` — every version + its verification status.
- Tauri v2 API shapes used here were checked against 2.12.5 docs; each is listed as VERIFIED in
  `docs/windows-port/21-mvp-log.md` §5.

## Amendment 2 — setup guidance + minimal persistence (2026-11-15, accepted by the mission prompt)

The MVP grows a second job: **when the SDK is missing or incomplete, guide the user — never install
anything.** This is a scope change to ADR-0007, not a new architecture.

### Decisions

1. **Detect + guide, never install.** Detection is read-only: `Path::is_file` / `is_dir` on Studio, SDK,
   `platform-toolsdb.exe`, `emulator.exe`, and a count of `*.ini` in the AVD folder. The *only* process
   run for setup is `adb version` (a client-local command; it does **not** start the daemon — verified,
   `21-mvp-log.md` §7). The app never downloads, never launches Studio/emulator, never writes env vars.
   Rationale: installing software on a user's behalf is a different product (and a different licence and
   security surface); the upstream app also only reads `AndroidSDK.swift:13-36`.
2. **AVD detection reads the folder, it does not spawn the emulator.** `%USERPROFILE%\.androidvd\*.ini`
   is exactly what `emulator -list-avds` enumerates; spawning the emulator to answer "is one configured?"
   would violate decision 1 and cost seconds (`EmulatorLauncher.swift:8-16` launches it for real).
3. **New plugins, both narrowly scoped:** `tauri-plugin-dialog` (folder picker, `dialog:allow-open` only)
   and `tauri-plugin-opener` (`opener:allow-open-url` scoped to four exact `developer.android.com` URLs).
   No `shell`, no `fs`, no `store`, no `http`. Rationale: the app must be able to *point at* the official
   docs without embedding a browser or an HTTP client.
4. **Icons: Font Awesome Free, bundled offline.** `@fortawesome/fontawesome-free@7.3.1` via npm; Vite
   emits the woff2 into `dist/`. **No CDN**, no runtime network, CSP `font-src 'self'`. This supersedes the
   definitions-only approach of ADR-0008 — see **ADR-0009** for the measurement and the reversal.
5. **Minimal persistence exists now (override O3).** One key: `sdk.customRoot`. Config file
   `%APPDATA%\dev.bramburn.androidpanel\config.json`:

   ```json
   { "schemaVersion": 1, "sdk": { "customRoot": null } }
   ```

   - Written **only** by `src-tauri` (Rust), atomically (temp file + `rename` in the same directory).
   - Missing file = defaults, no error. Corrupt file = copy to `config.json.bak`, return defaults, and
     surface a warning flag in the UI. Never crash, never silently overwrite.
   - Schema is versioned from day one so a future field does not become a guess.

### Migration policy (binding)

Current schema is **1**, and `migrate()` has exactly one arm: `1 => Ok`. Any future bump ships a numbered
migration function **with tests**. Migrations are **executed only in the main repository location after
merge-back, and only with human approval** — never inside a worktree, never automatically on a branch that
has not been reviewed. The runbook is in `docs/windows-port/21-mvp-log.md` §4.

### Dependency table (one line each)

| Dependency | Why it is allowed |
|---|---|
| `tauri`, `@tauri-apps/api` | the chosen framework (ADR-0001) |
| `tauri-plugin-dialog` | native folder picker; no web-file-system workaround, no `shell` |
| `tauri-plugin-opener` | opens the official docs in the user's browser; scoped to 4 exact URLs |
| `@fortawesome/fontawesome-free` | offline monochrome icons; attribution in `NOTICE.md` (ADR-0009) |
| `tokio` (`process`, `time`) | async child processes with timeouts |
| `serde`, `serde_json` | IPC + the versioned config file |
| `thiserror` | `AdbError` without hand-written `Display` |
| `windows-sys` | Job Object (`KILL_ON_JOB_CLOSE`) and `CREATE_NO_WINDOW` |
| `svelte`, `vite`, `typescript`, `svelte-check` | UI build + type gate |
