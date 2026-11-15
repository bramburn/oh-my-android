# Oh My Android — a **Windows-only** fork

A fork of **[Oh My Android](https://github.com/ateymoori/oh-my-android)** (MIT, © 2026 Royan AB), a native
macOS menu-bar GUI and embedded MCP server for `adb`-driven Android testing.

> ## This fork targets Windows exclusively.
>
> - The ship target is a **Windows 10/11 x64 desktop app**: **Tauri v2 + Rust + WebView2 + Svelte 5**, packaged
>   as a per-user **NSIS installer**.
> - The upstream **Swift 6 / SwiftUI / AppKit (macOS 26+)** sources under [`Sources/`](Sources/) are kept
>   **verbatim and unmodified** as a *reference implementation* — this fork does not build, maintain or ship
>   the macOS app.
> - The **MCP server is deliberately not ported.** This fork is an app for humans, not an agent backend.
> - No Linux, no mobile client, no cross-platform abstraction layer. Windows is the only supported OS.

| | |
|---|---|
| Status | **Windows MVP implemented** — setup guide + SDK picker, device list, adb server start/stop, NSIS installer |
| Windows code | [`crates/adbkit/`](crates/adbkit/) · [`src-tauri/`](src-tauri/) · [`src/`](src/) |
| Plan and decisions | [`docs/windows-port/`](docs/windows-port/) · [`docs/adr/`](docs/adr/) |
| Fork point | `9f37391eef056c0e6a45327b56670f200bb62460` (upstream `main`, release `v0.4.6`) |
| Baseline tag | `upstream-baseline-9f37391` |
| Working branch | `feat/win-mvp-tauri` |
| Upstream / origin | `upstream` → `ateymoori/oh-my-android`, `origin` → `bramburn/oh-my-android` |
| Upstream docs | preserved verbatim in [`docs/upstream/`](docs/upstream/) |

## What works today (Windows MVP)

One monochrome window. It tells you whether your Android tooling is usable, lets you point it at an SDK
folder, and then lists the devices `adb devices -l` reports.

| Capability | State |
|---|---|
| **Setup guide** — detects Android Studio, the SDK, `adb`, the emulator and AVDs; links the official docs | ✅ shown automatically **only when `adb` is missing** |
| Choose your SDK folder (native picker, validated, remembered) | ✅ requires `platform-tools\adb.exe` |
| List connected devices (serial, model, state) | ✅ refresh-only — **no hotplug** (override O1) |
| Start adb server / Stop adb server | ✅ `Stop` asks for confirmation (override O2) |
| SDK discovery: saved folder → `ANDROID_HOME` → `ANDROID_SDK_ROOT` → `%LOCALAPPDATA%\Android\Sdk` → legacy `%ProgramFiles(x86)%` | ✅ the app ships **no** `adb` |
| Remembers your SDK folder across restarts | ✅ one key, versioned JSON, atomic write |
| NSIS installer, per-user install, silent install/uninstall | ✅ ~1.6 MiB |
| Everything else in the upstream product | ⛔ **not implemented yet** — see the parity plan |

This is a **scaffold with a real vertical slice**, not feature parity. Upstream's appearance/accessibility/
layout/app/data/simulate panels and the MCP server are all still ahead
([`docs/windows-port/04-feature-parity-plan.md`](docs/windows-port/04-feature-parity-plan.md),
[`docs/feature-matrix.md`](docs/feature-matrix.md)). Known gaps and every measurement are in
[`docs/windows-port/21-mvp-log.md`](docs/windows-port/21-mvp-log.md) — including the honest `UNVERIFIED` list.

---

# Build, install, use

## 1. Prerequisites

| Requirement | Why | How |
|---|---|---|
| **Windows 10/11 x64** | the only supported target | — |
| **WebView2 runtime** | the UI is HTML/CSS inside WebView2 | preinstalled on Windows 11; on Windows 10 the installer fetches it (`downloadBootstrapper`, silent) |
| **Rust 1.90** | compiles `crates/adbkit` + `src-tauri` | [rustup](https://rustup.rs); the exact channel is pinned in [`rust-toolchain.toml`](rust-toolchain.toml) and installed automatically |
| **Node 24** (`.nvmrc` = `24.11.0`) | builds the Svelte frontend | [nodejs.org](https://nodejs.org) or `fnm` / `nvm-windows` |
| **MSVC C++ build tools** | the Rust linker on Windows | "Desktop development with C++" in the Visual Studio Build Tools installer |
| **Android SDK `platform-tools`** | provides `adb.exe` — **the app does not bundle it** | install *Android Studio* + SDK, or download "SDK Platform-Tools" alone; then set `ANDROID_HOME` or point the app at the folder from its setup screen |

Optional: an emulator/AVD (Studio → SDK Manager → Android Emulator) or a physical phone with **USB debugging**
enabled.

Check the toolchain before building:

```powershell
rustc --version    # rustc 1.90.0
node --version     # v24.11.0
cargo --version
```

## 2. Build

```powershell
git clone https://github.com/bramburn/oh-my-android.git
cd oh-my-android
npm ci                # exact lockfile install (use `npm install` only when changing dependencies)
```

### Development run (hot reload)

```powershell
npm run tauri dev
```

Vite serves the UI on `http://localhost:1420` and Rust rebuilds on change. The first build compiles the whole
dependency tree (a few minutes); later builds take seconds.

### Production build → installer

```powershell
npm run tauri build
```

Output:

```
target\release\android-panel.exe                                              # the app (~4.8 MB)
target\release\bundle\nsis\Android Panel (working title)_0.1.0_x64-setup.exe    # the installer (~1.6 MiB)
```

Both are unsigned (no code-signing certificate yet), so SmartScreen warns on first run — see
[`docs/windows-port/16-build-package-release.md`](docs/windows-port/16-build-package-release.md). The version
comes from `src-tauri/tauri.conf.json`; bump it there before a release.

If a build looks stale, drop the generated state and rebuild:

```powershell
Remove-Item -Recurse -Force dist, target\release\bundle
npm run tauri build
```

### Where the code lives

| Path | Role |
|---|---|
| `crates/adbkit/` | **The only code allowed to spawn a process.** Device-list parser, process runner (`CREATE_NO_WINDOW`, job objects, timeouts), SDK locator, setup detection |
| `src-tauri/` | Tauri v2 shell: six thin commands (`setup_report`, `choose_sdk_folder`, `list_devices`, `adb_start`, `adb_stop`, `open_docs`), config store, `AppError`, window + CSP + NSIS config |
| `src/` | Svelte 5 + TypeScript UI, monochrome: `SetupScreen.svelte` (guide) + `DevicePanel.svelte` (devices); `src/lib/ipc.ts` is the only IPC surface |
| `Sources/` | **Read-only reference**: the upstream Swift implementation. Never edited from this fork |
| `docs/windows-port/` | Target, version pins, IPC contract, frontend design, build/release, testing, backlog, risk register |
| `docs/adr/` | Accepted decisions (Tauri, repo strategy, MVP scaffold, Font Awesome icon font) |

Design rules that are enforced, not aspirational: monochrome UI, argv-not-shell for every `adb` call, adb only
through `crates/adbkit` (CI greps for `Command::new` elsewhere), offline-first (CI rejects remote asset URLs),
Windows-only, no telemetry.


## 3. Install

### Normal install

Double-click `Android Panel (working title)_0.1.0_x64-setup.exe`, or from PowerShell:

```powershell
& ".\Android Panel (working title)_0.1.0_x64-setup.exe"
```

- **Per-user** install (`installMode: currentUser`): **no admin prompt**, no elevation.
- Destination: `%LOCALAPPDATA%\Android Panel (working title)\` (`android-panel.exe` + `uninstall.exe`).
- Creates a Start-menu entry and an "Apps & features" uninstall entry.
- If WebView2 is missing, the installer downloads and installs it silently first (needs internet **once**).

### Silent install (scripts / CI)

```powershell
& ".\Android Panel (working title)_0.1.0_x64-setup.exe" /S
```

### Uninstall

Settings → Apps → *Android Panel (working title)* → Uninstall, or:

```powershell
& "$env:LOCALAPPDATA\Android Panel (working title)\uninstall.exe" /S
```

Your settings file is **not** removed by the uninstaller. Clean it by hand for a fresh start:

```powershell
Remove-Item "$env:APPDATA\dev.bramburn.androidpanel" -Recurse -Force      # settings
Remove-Item "$env:LOCALAPPDATA\dev.bramburn.androidpanel" -Recurse -Force # WebView2 cache
```

## 4. Use it

Start **Android Panel**. There are exactly two screens, and which one you get is not a setting — it is a fact
about your machine.

### First run, `adb` not found → the setup guide

The guide opens by itself, because without `adb` there is nothing else the app can do. It reports, top to
bottom, what it actually found on disk:

| Row | ✓ | ✗ / info |
|---|---|---|
| **Android Studio** | path to `studio64.exe` | link to the download page |
| **Android SDK (platform-tools)** | SDK root + *how it was found* (saved folder, `ANDROID_HOME`, default location…) | **Choose SDK folder…** + platform-tools docs |
| **adb** | version string reported by `adb version` | explanation when `adb.exe` exists but does not answer |
| **Emulator** | path to `emulator.exe` | not needed for a physical phone |
| **Virtual devices** | how many AVDs are configured | link to the AVD docs |

To fix a missing SDK:

1. Press **Choose SDK folder…** and pick the folder that **contains** `platform-tools\`
   (typically `%LOCALAPPDATA%\Android\Sdk`) — not `platform-tools` itself, and not `adb.exe`.
2. The folder is validated immediately. A wrong folder is rejected with
   `No platform-tools\adb in <path>.` and nothing is saved.
3. The guide re-renders with the new result, and the choice is remembered for next time.
4. **Continue to devices** becomes available only once a runnable `adb` exists.

**Re-check** re-runs detection without restarting the app — use it after installing the SDK or Studio.

Every external link (**Open download page**, **Open platform-tools docs**, **Open AVD docs**, **Open USB
debugging guide**) opens your default browser. The app never loads a web page itself and works with the
network disabled.

### `adb` found → the device panel

The guide is skipped entirely and you land on the device list:

- **Device rows** — model name, serial, state badge: `READY` (`device`), `UNAUTHORIZED` (accept the USB
  debugging prompt on the phone), `OFFLINE` (booting or a stale connection), `OTHER`. Emulators show a monitor
  icon, physical devices a phone icon.
- **Refresh** — re-runs `adb devices -l`. This is the **only** way the list changes: there is no hotplug
  watching yet (override O1), so plugging in a phone requires a click.
- **Start adb** — runs `adb start-server` (harmless when the server is already up).
- **Stop adb** — runs `adb kill-server`, which disconnects **every** adb client on this machine, so it asks
  for confirmation first. Never invoked automatically.
- **Wrench button** (top right) — reopens the setup guide at any time, e.g. to see which SDK folder is in use
  or to switch to another one.

Other states: `Checking for devices…` (first load), `No devices connected.` (empty list — plug in a phone or
start an emulator, then Refresh), and `Could not read devices` with the exact error and a **Retry** button. A
failed refresh never leaves stale rows on screen.

Phone plugged in but the row says `UNAUTHORIZED`? Unlock it, accept the "Allow USB debugging?" prompt, then
press **Refresh**.

### Where your settings live

| Path | Contents |
|---|---|
| `%APPDATA%\dev.bramburn.androidpanel\config.json` | `{"schemaVersion": 1, "sdk": {"customRoot": "..."}}` — your SDK folder choice, nothing else |
| `%LOCALAPPDATA%\dev.bramburn.androidpanel\` | WebView2 user data (cache); safe to delete while the app is closed |

The file is written atomically. If it is ever corrupted, or written by a **newer** schema, the app copies it
to `config.json.bak`, starts from defaults and shows a warning banner — it never guesses at an unknown format
and never crashes on a bad file.

### What the app does *not* do

No installs, no downloads, no bundled `adb`, no device modification, no telemetry, no auto-update. The only
processes it runs are `adb version` (read-only probe), `adb devices -l`, `adb start-server` and
`adb kill-server` — always as argv arrays, never through a shell.

## 5. Verify a build (gates)

All of these must pass, and do pass on Windows 11 / WebView2 157:

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace        # 46 tests: 23 adbkit unit + 8 parser golden + 15 src-tauri
npm run check                 # svelte-check
npm run build                 # type-check + Vite production build
```

CI for the Windows app: [`.github/workflows/windows.yml`](.github/workflows/windows.yml) — fmt, clippy, tests,
three guards (no `Command::new` outside `crates/adbkit`, no Tauri v1 APIs, no remote asset URLs), NSIS build,
artifact upload. The upstream macOS workflow [`.github/workflows/build.yml`](.github/workflows/build.yml) is
**left untouched** so the reference implementation keeps compiling on its own terms.

## 6. Troubleshooting

| Symptom | Cause / fix |
|---|---|
| Guide reappears every launch | the chosen folder has no `platform-tools\adb.exe`, or `config.json` is unreadable. Re-pick the folder and check for a `config.json.bak` banner |
| `Continue to devices` stays unavailable | no *runnable* `adb`: the SDK row can be ✓ while the `adb` row is ✗ (blocked by antivirus, corrupted download) → reinstall platform-tools |
| App shows no devices, but `adb devices` in a terminal lists one | the app is using a **different** `adb.exe`. Compare with the path shown in the setup guide (wrench button) |
| Device shows `UNAUTHORIZED` | accept the RSA prompt on the phone, press Refresh. If it never appears, revoke USB debugging authorizations in Developer options |
| Device shows `OFFLINE` right after **Start adb** | the server just restarted; wait a second and press Refresh |
| `npm run tauri build` fails in the Rust step | the frontend is embedded at compile time: run `npm run build` first (CI does), and confirm the MSVC build tools are installed |
| Installer warns about an unknown publisher | builds are unsigned; expected until a certificate is configured |

## Why a port at all

Upstream is a native macOS menu-bar panel: **Swift 6 + SwiftUI/AppKit, macOS 26+**, with a second executable
target (`ohmyandroid-mcp`) embedded in the app bundle. It is not Electron, not Tauri, not .NET. It has **no
screen mirroring** and **no bundled `adb`**. None of that ports to Windows by compilation, so the GUI was
re-implemented against the same product requirements while the Swift sources stay as the behavioural
specification — see [`docs/repo-inventory.md`](docs/repo-inventory.md) and
[`docs/adr/ADR-0001-framework-tauri.md`](docs/adr/ADR-0001-framework-tauri.md).

For context, the upstream product (macOS, and the eventual parity target for this fork) does:

- **Appearance** — dark mode, font size, display size, language/locale incl. RTL and pseudo-locales.
- **Accessibility** — TalkBack, bold text, invert colors, and a TalkBack-order audit with missing-label and
  48 dp touch-target checks.
- **Layout** — a dp measurement inspector with design-overlay comparison, layout bounds, overdraw, GPU bars.
- **App** — restart, clear data, force stop, uninstall, reset permissions, deep links, APK install.
- **Data** — read-only SharedPreferences and SQLite of debuggable builds.
- **Simulate** — network profiles, GPS, battery, incoming call/SMS (emulator console).
- **Agents** — an MCP server with 18 tools and 3 prompts over stdio — **dropped in this fork**.

## Documentation index

Start with [`AGENTS.md`](AGENTS.md) (invariants and file map) and
[`docs/windows-port/README.md`](docs/windows-port/README.md) (reading order).

| Document | What it answers |
|---|---|
| [`docs/windows-port/00-target-and-constraints.md`](docs/windows-port/00-target-and-constraints.md) | Scope, budgets, naming placeholders |
| [`docs/windows-port/10-version-pin.md`](docs/windows-port/10-version-pin.md) | Every version actually used, with sources |
| [`docs/windows-port/11-tauri-architecture.md`](docs/windows-port/11-tauri-architecture.md) | Workspace layout, modules, threading, Windows process rules |
| [`docs/windows-port/12-ipc-contract.md`](docs/windows-port/12-ipc-contract.md) | Command/event table and error taxonomy |
| [`docs/windows-port/14-frontend-design.md`](docs/windows-port/14-frontend-design.md) | Component tree, state, monochrome design, budgets |
| [`docs/windows-port/16-build-package-release.md`](docs/windows-port/16-build-package-release.md) | Bundling, signing, CI |
| [`docs/windows-port/18-plan-and-backlog.md`](docs/windows-port/18-plan-and-backlog.md) | Milestones and task backlog |
| [`docs/windows-port/21-mvp-log.md`](docs/windows-port/21-mvp-log.md) | **What was built, measured, deviated and left unverified** |
| [`docs/architecture/06-adb-integration.md`](docs/architecture/06-adb-integration.md) | Every upstream adb call, cited (the behavioural spec) |
| [`docs/feature-matrix.md`](docs/feature-matrix.md) | Every feature with complexity, Windows feasibility, MVP tier |
| [`docs/git-setup.md`](docs/git-setup.md) · [`docs/legal.md`](docs/legal.md) | Remotes/branches · licence obligations and naming |
| [`docs/CONTRIBUTING-fork.md`](docs/CONTRIBUTING-fork.md) | Fork conventions, ADB safety rules, PR checklist |

Upstream's own maps are kept: [`docs/repo-inventory.md`](docs/repo-inventory.md),
[`docs/coverage.md`](docs/coverage.md), [`docs/assumptions.md`](docs/assumptions.md),
[`docs/open-questions.md`](docs/open-questions.md),
[`docs/architecture/00-overview.md`](docs/architecture/00-overview.md).

## Building the original macOS app (reference only)

Kept for reading the reference implementation; this fork does not support or ship it.

```sh
brew install xcodegen
xcodegen generate
xcodebuild -project OhMyAndroid.xcodeproj -scheme OhMyAndroid -configuration Release \
  -derivedDataPath build/DerivedData CODE_SIGN_IDENTITY=- build
```

`UNVERIFIED` on this fork's machine (no macOS, no Xcode).

## Safety

The Windows app runs exactly four adb invocations, all through `crates/adbkit` with argv arrays (never a shell
string): `adb version` (read-only probe), `adb devices -l`, `adb start-server`, `adb kill-server`.
`kill-server` disconnects every client on the machine, so it sits behind a confirmation dialog and is never
invoked automatically or from a test. Quitting the app also tears the adb daemon down (measured; see
`21-mvp-log.md` §3 D5). Rules for future work are in [`docs/CONTRIBUTING-fork.md`](docs/CONTRIBUTING-fork.md)
and [`AGENTS.md`](AGENTS.md).

## Licence

MIT, as upstream. See [`LICENSE`](LICENSE) (unchanged) and [`NOTICE.md`](NOTICE.md) — which also carries the
**required CC BY 4.0 / SIL OFL 1.1 attribution for the Font Awesome icon font** bundled in the Windows app.
"Oh My Android", the bundle identifier `se.royan.ohmyandroid`, the app icon and the Sparkle feed are upstream
identity and are **not** reused here: the Windows app ships under a placeholder name and its own generated
monochrome icon.