# 21 — MVP log (Windows scaffold)

Living record for the `feat/win-mvp-tauri` branch. Newest section last.

## 1. Scope

Three commands — `list_devices`, `adb_start`, `adb_stop` — a monochrome device list, and an NSIS
installer. Everything else is out (`docs/adr/ADR-0007-win-mvp-scaffold.md`).

## 2. Overrides of `AGENTS.md`

| # | Override | Status |
|---|---|---|
| **O1** | Device list updates on explicit **Refresh** only. Invariant #4 (event-driven hotplug, `Sources/Core/Android/DeviceTracker.swift:96-118`) is **deferred**. | Active for this MVP |
| **O2** | **Start/Stop targets the adb server** (`adb start-server` / `adb kill-server`), not emulators or devices. `kill-server` is on the human-approval list (`AGENTS.md` §7); the button + `confirm()` dialog *is* the approval. Never automatic, never in a test. | Active for this MVP |

Consequence of O1 that matters: an error must not leave stale rows on screen — the list is cleared before
the state is set to `error`.

## 3. Deviations from the brief

| # | Brief | What was done | Why |
|---|---|---|---|
| D1 | (record after implementation) | | |

## 4. Persistence and migration rule (encoded now, used later)

- **There is no database and no migration in this MVP.** Upstream persists only preferences
  (`docs/architecture/04-data-and-state.md`); this MVP persists **nothing** — the SDK path is re-located on
  every command by `adbkit::sdk::locate`.
- Any future persisted store is JSON with `"schemaVersion": 1`. Bump on a breaking shape change.
- Migration scripts are committed as files but **executed only in the main repo checkout, only after the
  merge, only with human approval**. Never in the worktree, never by the agent.
- Merge-back (run by the human, not the agent):

```powershell
cd C:\dev\oh-my-android
git merge --no-ff feat/win-mvp-tauri -m "merge: win MVP tauri scaffold"
# migrations: none for this MVP. If a future merge adds one, run it HERE, now, not before.
git worktree remove C:\dev\worktrees_tmp\oma-win-mvp
git branch -d feat/win-mvp-tauri   # optional, after verification
```

## 5. Verification status of every API used

| API | Status |
|---|---|
| `tokio::process::Command::creation_flags` (Windows) | VERIFIED by compilation — `cargo build` on 1.89.0 / MSVC |
| `tokio::process::Child::raw_handle` (Windows) | VERIFIED by compilation |
| `windows_sys::Win32::System::JobObjects::*` names | VERIFIED by compilation |
| `tauri::Builder` / `generate_handler!` / `#[tauri::command] async fn` with `State<'_, T>` | VERIFIED by compilation + `npm run tauri build` |
| `app.security.csp`, `bundle.targets`, `nsis.installMode`, `webviewInstallMode` keys | VERIFIED against https://schema.tauri.app/config/2 |
| WebView2 runtime present on this machine | VERIFIED — `157.0.4341.57` (registry `pv`) |
| `adb` present on this machine | VERIFIED — `1.0.41` / `37.0.1-15733141` at `%LOCALAPPDATA%\Android\Sdk\platform-tools\adb.exe` |
| MSVC link step | VERIFIED — a scratch `cargo new` binary links (`_linktest.exe`) |
| WebCodecs / video | N/A — out of MVP scope |

## 6. Measurements

| Metric | Value |
|---|---|
| (recorded after T5) | |

## 7. UNVERIFIED items

- Anything not exercised on a second machine: clean Windows 10 22H2 install, offline `downloadBootstrapper`
  behaviour, SmartScreen (unsigned), a second monitor / 200 % DPI.
- `.github/workflows/windows.yml` has not run on GitHub (no push).
- Real-device behaviour beyond `adb devices`/`start-server`/`kill-server`, which were exercised manually.

## 8. Next milestone proposal

1. `host:track-devices` + reconnect/backoff, retiring override O1 (`11-tauri-architecture.md` §3).
2. The 8-state panel (`Sources/State/AppModel.swift:47-70`, `docs/architecture/08-ui-ux-map.md` §3) replacing
   the MVP's 5 states.
3. `crates/fake-adb` + golden fixtures for every parser (`17-testing-strategy.md` §3-4).
4. Feature engine per `docs/feature-matrix.md` P0 list, starting with appearance toggles.