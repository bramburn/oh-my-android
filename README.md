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
| Status | **Windows MVP implemented** — device list + adb server start/stop, NSIS installer |
| Windows code | [`crates/adbkit/`](crates/adbkit/) · [`src-tauri/`](src-tauri/) · [`src/`](src/) |
| Plan and decisions | [`docs/windows-port/`](docs/windows-port/) · [`docs/adr/`](docs/adr/) |
| Fork point | `9f37391eef056c0e6a45327b56670f200bb62460` (upstream `main`, release `v0.4.6`) |
| Baseline tag | `upstream-baseline-9f37391` |
| Working branch | `feat/windows-gui-port` |
| Upstream / origin | `upstream` → `ateymoori/oh-my-android`, `origin` → `bramburn/oh-my-android` |
| Upstream docs | preserved verbatim in [`docs/upstream/`](docs/upstream/) |

## What works today (Windows MVP)

The app is a single monochrome window: it lists the devices `adb devices -l` reports and can start/stop the
local adb server.

| Capability | State |
|---|---|
| List connected devices (serial, model, state) | ✅ refresh-only — **no hotplug** (override O1) |
| Start adb server / Stop adb server | ✅ `Stop` asks for confirmation (override O2) |
| SDK discovery (`ANDROID_HOME`, `ANDROID_SDK_ROOT`, standard install paths) | ✅ app ships **no** `adb` |
| NSIS installer, per-user install, silent install/uninstall | ✅ ~1.3 MiB |
| Everything else in the upstream product | ⛔ **not implemented yet** — see the parity plan |

This is a **scaffold with a real vertical slice**, not feature parity. Upstream's appearance/accessibility/
layout/app/data/simulate panels and the MCP server are all still ahead
([`docs/windows-port/04-feature-parity-plan.md`](docs/windows-port/04-feature-parity-plan.md),
[`docs/feature-matrix.md`](docs/feature-matrix.md)). Known gaps and every measurement are in
[`docs/windows-port/21-mvp-log.md`](docs/windows-port/21-mvp-log.md) — including the honest `UNVERIFIED` list.

## Build and run the Windows app

Requirements: **Windows 10/11 x64**, [WebView2 runtime](https://developer.microsoft.com/microsoft-edge/webview2/)
(preinstalled on Win11), **Rust 1.90** (pinned in `rust-toolchain.toml`, installed automatically by `rustup`),
**Node 24** (`.nvmrc`), and an Android SDK providing `platform-tools/adb.exe`
(set `ANDROID_HOME`, or install it in the default location). The installer does **not** bundle `adb`.

```powershell
npm install
npm run tauri dev      # dev window with hot reload
npm run tauri build    # → target\release\bundle\nsis\*_x64-setup.exe
```

Gates (all verified on Windows 11 / WebView2 157):

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace     # 28 tests: no device, no real adb
npm run check              # svelte-check
npm run build              # vite
```

CI for the Windows app: [`.github/workflows/windows.yml`](.github/workflows/windows.yml) (fmt, clippy, tests,
two guard greps, NSIS build, artifact upload). The upstream macOS workflow
[`.github/workflows/build.yml`](.github/workflows/build.yml) is **left untouched** so the reference
implementation keeps compiling on its own terms.

### Where the code lives

| Path | Role |
|---|---|
| `crates/adbkit/` | **The only code allowed to spawn a process.** Device-list parser, process runner (`CREATE_NO_WINDOW`, job objects, timeouts), SDK locator |
| `src-tauri/` | Tauri v2 shell: three thin commands (`list_devices`, `adb_start`, `adb_stop`), `AppError`, window + CSP + NSIS config |
| `src/` | Svelte 5 + TypeScript UI, monochrome, one screen, `src/lib/ipc.ts` as the only IPC surface |
| `Sources/` | **Read-only reference**: the upstream Swift implementation. Never edit it from this fork |
| `docs/windows-port/` | Target, version pins, IPC contract, frontend design, build/release, testing, backlog, risk register |
| `docs/adr/` | Accepted decisions (Tauri, repo strategy, MVP scaffold, Font Awesome icons) |

Design rules that are enforced, not aspirational: monochrome UI, argv-not-shell for every `adb` call, adb only
through `crates/adbkit` (CI greps for `Command::new` elsewhere), Windows-only, no telemetry.

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

The Windows app runs exactly three adb invocations, all through `crates/adbkit` with argv arrays (never a shell
string): `adb devices -l`, `adb start-server`, `adb kill-server`. `kill-server` disconnects every client on the
machine, so it is behind a confirmation dialog and is never invoked automatically or from a test. Killing the
app also tears the adb daemon down (measured; see `21-mvp-log.md` §3 D5). Rules for future work are in
[`docs/CONTRIBUTING-fork.md`](docs/CONTRIBUTING-fork.md) and [`AGENTS.md`](AGENTS.md).

## Licence

MIT, as upstream. See [`LICENSE`](LICENSE) (unchanged) and [`NOTICE.md`](NOTICE.md) — which also carries the
**required CC BY 4.0 attribution for the Font Awesome icons** in the Windows app. "Oh My Android", the bundle
identifier `se.royan.ohmyandroid`, the app icon and the Sparkle feed are upstream identity and are **not** reused
here: the Windows app ships under a placeholder name and its own generated monochrome icon.