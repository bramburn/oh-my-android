# 00 — Target and constraints

## 1. Approved scope (`DECISION`, from Gate 1 + Gate 3)

| # | Decision | Consequence |
|---|---|---|
| D1 | **Windows-only GUI.** No macOS/iOS/Android/Linux target in the new code. | No `#[cfg(unix)]` fallbacks, no cross-platform abstraction "for later". |
| D2 | **Tauri v2** (Rust core + WebView2 + TypeScript UI). | Two languages, one IPC boundary, `docs/adr/ADR-0001`. |
| D3 | **App only — the MCP server is not ported.** | `Sources/MCP/**` and `Sources/Core/AgentAccess.swift` are reference-only. No `.mcpb`, no registry metadata, `Scripts/test-mcp.py` stays a macOS-only test. |
| D4 | **macOS CI and the macOS app are kept working.** | `.github/workflows/build.yml` untouched; a new `windows.yml` is added beside it. |
| D5 | **`Sources/**` is read-only reference** until the Windows app is built. | No edits, moves or deletions. New code lives in new top-level dirs (`05-repo-restructure.md`). |
| D6 | **Proper installer, not portable.** | NSIS per-user installer by default, MSI/WiX for enterprise; `16-build-package-release.md`. |
| D7 | **No live screen mirroring** — contested; see `ADR-0002`. | Plan is written so M0–M2 are identical either way. |
| D8 | No telemetry, ever. | Only network egress: `adb` + opt-in update check. |

## 2. Non-goals

Listing/diffing/recording/video playback · multi-device simultaneous control (MVP) · root/`remount` ·
reboot · plugin system · theming beyond dark/light · i18n beyond a scaffold · crash reporting ·
app-store distribution · ARM64 build in MVP (`ESTIMATE` +2 days later).

## 3. Hard budgets (`DECISION` unless marked)

| Budget | Value | Why |
|---|---|---|
| Installer size | **≤ 25 MB** including `platform-tools` | `platform-tools` folder is ~10 MB zipped; app+WebView2-bootstrapper ~6 MB |
| Cold start to interactive panel | **≤ 1.5 s** | a panel you wait for is a panel you stop using |
| Device list refresh after plug-in | **≤ 500 ms** | event-driven, not polled |
| Batched device read (all top-of-panel data) | **1 adb process** per refresh | mirrors upstream `ADBClient.swift:31-36` |
| Cancelled action → child process gone | **≤ 1 s** | Job Object kill |
| Idle CPU | **< 1 %** | no polling loops |
| Logcat UI | 20 000 lines/s ingested without jank | coalesced + virtualized |
| Mirror glass-to-glass (if M3 happens) | **≤ 120 ms** `SPIKE` | S3 pass criterion |
| Max IPC message rate | 60 msg/s per channel after coalescing | `12-ipc-contract.md` §5 |

## 4. Naming and identity

The fork must not reuse upstream identity (`docs/legal.md`). Placeholder values — **confirm before M1**:

| Field | Placeholder | Notes |
|---|---|---|
| Product name | `ADB Panel` | working title; needs a check for trademark collision |
| Binary / exe | `adb-panel.exe` | |
| Bundle identifier | `io.github.bramburn.adbpanel` | must not be `se.royan.ohmyandroid` |
| Updater key | *new* minisign keypair | never upstream's `SUPublicEDKey` (`project.yml:58`) |
| Updater feed | own GitHub releases | |
| Icon | new asset | upstream icons are `Sources/App/Assets.xcassets/**` — do not copy |

## 5. Environment on the machine that produced this plan

| Tool | Version | Note |
|---|---|---|
| `cargo` / `rustc` | 1.89.0 (2025-08-04) | `x86_64-pc-windows-msvc` assumed — `UNVERIFIED` (no `rustup target list` run) |
| `node` / `npm` / `bun` | 24.11.0 / 11.21.1 / 1.4.4 | Node 24 is the pin |
| `adb` | **not installed** | no device access was possible; nothing here was tested against hardware |
| macOS / Xcode | absent | upstream build never executed in this fork |

## 6. What "done" means for v1.0

Upstream parity **minus the MCP server and minus the Layout Inspector canvas** (tier P1 in
`../feature-matrix.md`), plus wireless debugging, which upstream lacks. Feature-level detail:
`04-feature-parity-plan.md`.