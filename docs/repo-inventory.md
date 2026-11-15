# Repo inventory

Phase 0 recon. Evidence-only. Anything not executed is marked `UNVERIFIED`.

Baseline: `main` @ `9f37391` ("docs: Android-first README intro"), tag `v0.4.6`, origin
`https://github.com/ateymoori/oh-my-android.git`.

## 1. Stack summary

| Question | Answer | Evidence |
|---|---|---|
| What is it | A native **macOS** control panel for Android emulators/phones over `adb`, plus an MCP server (stdio) for AI agents | `README.md:1-30`, `project.yml:1-90` |
| Language | Swift 6, strict concurrency, warnings-as-errors | `project.yml:10`, `project.yml:20` |
| UI toolkit | SwiftUI (`@Observable`, `MenuBarExtra`, `glassEffect`) + AppKit (`NSPanel`, `NSWorkspace`, `NSOpenPanel`) | `Sources/UI/PanelRootView.swift:1-40`, `Sources/Windows/FloatingPanel.swift:6-47` |
| Build system | XcodeGen (`project.yml`) → `xcodebuild`; the `.xcodeproj` is generated and git-ignored | `project.yml:1`, `.gitignore:1-2`, `.github/workflows/build.yml:23-30` |
| Package manager | SwiftPM (single dependency: Sparkle 2.10.0) | `project.yml:26-31` |
| Min OS | macOS **26.0** (Tahoe); Apple silicon only per README badge | `project.yml:4-5`, `project.yml:11`, `README.md:14` |
| App bundle id | `se.royan.ohmyandroid` (doubles as the preferences domain) | `project.yml:74`, `Sources/Core/AgentAccess.swift:37` |
| Second product | `ohmyandroid-mcp`, a `tool` target embedded in the app bundle at `Contents/MacOS/` | `project.yml:83-93` |
| Sandbox | Disabled (`com.apple.security.app-sandbox: false`) — needed to spawn the SDK's `adb` | `project.yml:69-72` |
| License | MIT, © 2026 Royan AB | `LICENSE:1-3` |
| LOC (Swift) | **6 605** lines across 71 `.swift` files | `wc -l` over `Sources/**/*.swift` |
| Third-party code | Sparkle.framework only; no vendored sources, no bundled `adb`/`platform-tools`/`scrcpy` | `project.yml:26-31`, `Sources/Core/Android/AndroidSDK.swift:3-6` |
| Telemetry | None. One network call: Sparkle appcast GET | `SECURITY.md:9-16`, `Sources/App/UpdateController.swift:16-30` |

### What this repo is *not* (important for the Windows-port plan)

- **Not** Electron, Tauri, .NET, Qt, Python or Java. There is no web UI to reuse.
- **No screen mirroring.** No `scrcpy`, no `app_process`, no `localabstract`, no H.264 anywhere:
  `rg -ni "scrcpy|app_process|localabstract|h264" Sources` returns one hit, and it is the word
  "mirrored" in an LLM prompt (`Sources/MCP/Server/Catalog.swift:40`). Vision is `screencap -p`
  snapshots only (`Sources/Core/Android/ADBClient.swift:21-27`).
- **No ADB library.** All device work shells out to `<sdk>/platform-tools/adb`, except one raw
  TCP call to the adb server (`host:track-devices`) for hotplug (`Sources/Core/Android/DeviceTracker.swift:96-118`).
- **No database, no cache dir, no migrations.** State is in-memory + `UserDefaults`/`CFPreferences`.
  The only SQLite use is read-only over a *local copy* of a device DB (`Sources/Core/SQLite/SQLiteSnapshot.swift:19-88`).
- **No i18n.** All strings are hard-coded English; `SWIFT_EMIT_LOC_STRINGS: NO` (`project.yml:79`).
- **No logcat streaming.** `logcat` is a one-shot `-d` dump with `tail` (`Sources/MCP/Tools/AppTools.swift:116-151`).
- **No push/pull file transfer, no wireless pairing, no `tcpip`, no `root`/`remount`, no `reboot`.**

## 2. Directory map

```
Sources/
  App/        SwiftUI App + AppDelegate, Settings, Sparkle updates, assets
  Core/       Android (adb/SDK/device/inspectors), Shell, SQLite, Imaging, AgentAccess
  Features/   One small type per user-facing capability + catalog
  MCP/        MCP stdio server: protocol, tools, prompts, formatting
  State/      @Observable stores (app model, devices, features, emulators, inspectors)
  UI/         Panel views, feature grid, inspectors
  Windows/    NSPanel controllers, emulator window tracking
Scripts/      make-icon.swift, release.sh, test-mcp.py
skills/, .claude-plugin/, mcpb/, server.json, glama.json   Agent-marketplace packaging
docs/         COMPARISON.md + screenshots only (no engineering docs before this fork)
```

## 3. File table

`first-party/vendored/generated` · `crit` = criticality · `W-port` = disposition for a Windows GUI.

| path | purpose | LOC | lang | kind | crit | W-port | notes |
|---|---|---|---|---|---|---|---|
| `project.yml` | Whole build definition (2 targets, Info.plist, entitlements, Sparkle pin) | 93 | YAML | first-party | H | Reference-only | Single source of truth for the app shape |
| `Sources/Core/Shell/ShellRunner.swift` | The only place a `Process` is created: async pipes, 60 s watchdog, cancellation, `runDetached` for daemons | 174 | Swift | first-party | H | Rewrite | Maps 1:1 to a Windows `ProcessSupervisor` (Job Objects, `CREATE_NO_WINDOW`) |
| `Sources/Core/Android/ADBClient.swift` | The adb choke point: `devices`, `shell`, `exec-out`, `emu`, `kill/start-server`, PNG extraction | 121 | Swift | first-party | H | Rewrite | Protocol `ADBClient` + one implementation |
| `Sources/Core/Android/DeviceTracker.swift` | Raw `host:track-devices` TCP socket on 5037, hotplug push + backoff reconnect | 120 | Swift | first-party | H | Rewrite | Only non-`Process` adb usage |
| `Sources/Core/Android/AndroidSDK.swift` | SDK discovery chain (`sdk.customRoot`, `ANDROID_HOME`, `ANDROID_SDK_ROOT`, Studio, Homebrew) | 36 | Swift | first-party | H | Rewrite | macOS paths hard-coded; Windows paths differ (`%LOCALAPPDATA%\Android\Sdk`) |
| `Sources/Core/Android/LayoutSnapshot.swift` | `uiautomator dump` → `UINode` tree + density; XML parsing; retry on animating screen | 155 | Swift | first-party | H | Rewrite | Core of the Layout Inspector and `get_ui` |
| `Sources/Core/Android/AccessibilityTraversal.swift` | Approximates TalkBack order, merges labels, flags <48 dp targets | 116 | Swift | first-party | H | Adapt | Pure logic, portable |
| `Sources/Core/Android/AppData.swift` | `run-as` reads of SharedPreferences/SQLite of debuggable apps; local copy of DB + WAL/SHM | 141 | Swift | first-party | H | Adapt | Shell scripts inside; quoting rules matter |
| `Sources/Core/SQLite/SQLiteSnapshot.swift` | Read-only SQL over the pulled copy, `PRAGMA query_only`, `ATTACH` disabled | 132 | Swift | first-party | M | Adapt | Windows: SQLite via `Microsoft.Data.Sqlite`/bundled amalgamation |
| `Sources/Core/Android/DeviceInfo.swift` | Device facts in **one** `adb shell` round-trip (9 commands, separator trick) | 87 | Swift | first-party | H | Adapt | Reference for the "batch read" pattern |
| `Sources/Core/Android/EmulatorLauncher.swift` | `emulator -list-avds`, `emulator -avd <name>`, detached | 38 | Swift | first-party | M | Rewrite | Windows path `emulator\emulator.exe` |
| `Sources/Core/Android/ForegroundAppReader.swift` | `dumpsys activity activities \| grep -m1 Resumed` | 15 | Swift | first-party | M | Adapt | Regex on `u<id> <pkg>/` |
| `Sources/Core/AgentAccess.swift` | Shared app↔MCP-server policy (Off/Read/Full) via `CFPreferences` | 67 | Swift | first-party | H | Rewrite | Needs an equivalent cross-process settings store on Windows (registry/JSON) |
| `Sources/Core/Support.swift` | `AppError`, `String.shellQuoted`, `wm size`/`wm density` parsers | 38 | Swift | first-party | H | Adapt | `shellQuoted` is the injection guard |
| `Sources/Features/Feature.swift` | Feature protocols (`Toggle`, `Stepper`, `Choice`, `Action`, `TextAction`, `BatchReadable`) + `DeviceContext` + `HostActions` | 172 | Swift | first-party | H | Rewrite | The extension seam of the whole app |
| `Sources/Features/*Features.swift` (10 files) | ~50 concrete capabilities as data + one shell string each | 1 100 | Swift | first-party | H | Adapt | Highest-value reuse: the *commands and parsers* port verbatim |
| `Sources/Features/FeatureCatalog.swift` | Registration + display order + per-device filtering | 85 | Swift | first-party | H | Adapt | |
| `Sources/Features/BatchReader.swift` | Reads all `BatchReadable` features in one shell call, per-feature parallel fallback | 40 | Swift | first-party | M | Adapt | |
| `Sources/State/AppModel.swift` | Composition root; `PanelState` enum covering every no-device situation | 96 | Swift | first-party | H | Rewrite | |
| `Sources/State/DeviceStore.swift` | Device list + selection + 4 s recovery loop while nothing is ready | 91 | Swift | first-party | H | Rewrite | |
| `Sources/State/FeatureStore.swift` | Feature values, busy set, optimistic toggles, toast, 5 s staleness | 171 | Swift | first-party | H | Rewrite | |
| `Sources/State/{DeviceInfo,Emulator,Inspector,LayoutInspector}Store.swift` | Per-window view models | 300 | Swift | first-party | M | Rewrite | |
| `Sources/UI/PanelRootView.swift` | The panel: header, device summary, state switch, drop target, hover-refresh | 127 | Swift | first-party | H | Rewrite | |
| `Sources/UI/FeatureGridView.swift`, `FeatureCell.swift` | 3-column grid; each cell renders one feature's live value | 260 | Swift | first-party | H | Rewrite | |
| `Sources/UI/Theme.swift` | Tokens: panel 236×700, 3 columns, 46 pt buttons, 26 pt corner | 19 | Swift | first-party | L | Adapt | Numbers worth keeping |
| `Sources/UI/Layout/LayoutInspectorView.swift` | dp measurement canvas, overlay image, TalkBack audit list, keyboard nudge | 319 | Swift | first-party | H | Rewrite | Largest single view |
| `Sources/UI/Layout/AccessibilityListView.swift` | Ordered stop list with issue badges | 1 000+ (dir) | Swift | first-party | M | Rewrite | |
| `Sources/UI/Inspector/InspectorView.swift` | SharedPreferences + SQLite browser | 133 | Swift | first-party | M | Rewrite | |
| `Sources/Windows/*.swift` (5) | `NSPanel` subclasses, window controllers, emulator window tracker | 420 | Swift | first-party | M | Drop/Rewrite | macOS windowing has no analogue; Windows uses Win32/WPF windows |
| `Sources/App/AppDelegate.swift` | Wires `HostActions` (open panel, pickers, clipboard, Finder) | 65 | Swift | first-party | H | Rewrite | |
| `Sources/App/UpdateController.swift` | Sparkle updater + menu-bar badge | 79 | Swift | first-party | L | Rewrite | Windows needs Velopack/NetSparkle/none |
| `Sources/App/SettingsView.swift`, `AgentSettingsView.swift` | SDK path, dock, launch-at-login, agent access | 236 | Swift | first-party | M | Rewrite | |
| `Sources/App/OhMyAndroidApp.swift` | `MenuBarExtra` + `Settings` scenes | 55 | Swift | first-party | M | Rewrite | |
| `Sources/MCP/main.swift` | stdio loop: LF-split, one JSON-RPC message per line | 45 | Swift | first-party | H | Rewrite | Protocol is portable; code is not |
| `Sources/MCP/Server/MCPServer.swift` | Dual-era JSON-RPC (2026-07-28 `_meta` + legacy `initialize`), concurrent requests, cancellation | 181 | Swift | first-party | H | Rewrite | Well-documented spec in code; a good port target |
| `Sources/MCP/Server/Catalog.swift` | Tool groups, 3 prompts, server instructions | 92 | Swift | first-party | H | Adapt | Content (prompts) is reusable verbatim |
| `Sources/MCP/Tools/*.swift` (8) | 18 tools; each declares effect/access, params, and one implementation | 1 000 | Swift | first-party | H | Adapt | Tool schemas + shell snippets are reusable |
| `Sources/MCP/Format/UITreeText.swift` | Compact `[ref] Class "text" @x,y wxh` rendering in dp | 102 | Swift | first-party | M | Adapt | Output format is agent-facing; keep byte-compatible |
| `Sources/MCP/Protocol/JSONValue.swift` | Enum JSON with `.null`, ints vs doubles | ~120 | Swift | first-party | M | Drop | Any JSON library replaces it |
| `Sources/Core/Imaging/*.swift` | Nearest-neighbour downscale + pixel sampling for 1 px = 1 dp screenshots | 2 files | Swift | first-party | M | Adapt | Pure math, portable |
| `Scripts/release.sh` | Build → Developer ID sign (inside-out Sparkle) → notarize → staple → zip → appcast → Homebrew cask → `.mcpb` | 130+ | zsh | first-party | M | Reference-only | Entire flow is Apple-specific |
| `Scripts/test-mcp.py` | Protocol conformance test; no device needed; runs in CI | 103 | Python | first-party | H | Adapt | Best existing test asset; portable |
| `Scripts/make-icon.swift` | Icon generator | n/a | Swift | first-party | L | Drop | |
| `.github/workflows/build.yml` | `macos-26`: xcodegen → xcodebuild (Release) → `test-mcp.py` | 34 | YAML | first-party | H | Rewrite | Windows CI needs `windows-latest` |
| `.github/workflows/downloads.yml` | Shields.io download counter on the `badges` branch | 45 | YAML | first-party | L | Drop | Optional badge plumbing |
| `mcpb/manifest.json`, `server.json`, `.claude-plugin/*`, `glama.json`, `skills/android-ui-check/SKILL.md` | Agent-marketplace packaging; claims `compatibility.platforms: ["darwin"]` | — | JSON/MD | first-party | L | Adapt | Must add `win32` if MCP server is ported |
| `docs/*.jpg|gif|png` | Screenshots for README (2 MB GIF) | — | binary | first-party | L | Keep | Repo weight, not code |
| `LICENSE`, `SECURITY.md`, `CONTRIBUTING.md`, `docs/COMPARISON.md` | MIT + policy + code rules | — | MD | first-party | M | Keep | `CONTRIBUTING.md:20-32` states the architectural rules |

No vendored, generated, or build outputs are committed (only the icon PNGs in `Assets.xcassets`).

## 4. Top 15 must-read files, in order

1. `project.yml` — the whole product shape in one file (`project.yml:1-93`).
2. `CONTRIBUTING.md` — the author's stated architectural rules: `UI → State → Features → Core`, no polling, one `Process` seam, quote user text (`CONTRIBUTING.md:18-32`).
3. `Sources/Core/Android/ADBClient.swift` — the single adb choke point (`:4-15` protocol, `:39-121` impl).
4. `Sources/Core/Shell/ShellRunner.swift` — concurrency/timeout/cancellation contract for every adb call (`:32-113`).
5. `Sources/Core/Android/DeviceTracker.swift` — the only raw adb host-protocol usage (`:96-118`).
6. `Sources/Features/Feature.swift` — the capability model (`:71-172`).
7. `Sources/Features/FeatureCatalog.swift` — the complete feature list (`:5-71`).
8. `Sources/State/FeatureStore.swift` — batching, optimistic UI, staleness (`:27-154`).
9. `Sources/State/AppModel.swift` — composition root + `PanelState` (`:7-95`).
10. `Sources/State/DeviceStore.swift` — selection persistence + recovery loop (`:30-90`).
11. `Sources/MCP/Server/MCPServer.swift` — protocol behaviour that clients depend on (`:38-153`).
12. `Sources/MCP/Tools/Tool.swift` — tool contract + access gate (`:6-14`, `:129-200`).
13. `Sources/MCP/Tools/AppTools.swift:116-151` — the logcat implementation (the messiest shell script in the repo).
14. `Sources/Core/Android/LayoutSnapshot.swift` — how UI trees are obtained (`:82-109`).
15. `Scripts/release.sh` — what "shipping" means here (`:23-120`).

## 5. Commands that exist (and what was verified)

| Command | Where | Status here |
|---|---|---|
| `brew install xcodegen` | `CONTRIBUTING.md:9` | not run (macOS-only) |
| `xcodegen generate` | `CONTRIBUTING.md:10`, `.github/workflows/build.yml:25` | not run |
| `xcodebuild -project OhMyAndroid.xcodeproj -scheme OhMyAndroid -configuration Release ... build` | `.github/workflows/build.yml:28-30` | not run |
| `Scripts/test-mcp.py <binary>` | `Scripts/release.sh:59`, `.github/workflows/build.yml:31` | not run |
| `TEAM_ID=… Scripts/release.sh` | `Scripts/release.sh:9` | not run |
| `adb version` | — | **not run**: `adb` is not on PATH on this machine |

No `Makefile`, no `Package.swift`, no `Dockerfile`, no test target. There is no unit-test suite:
the only automated test is the Python MCP protocol check.

## 6. Windows-port implications (first pass only; full plan at Gate 2)

- Reusable **as text, not as code**: ~50 shell command strings + their parsers, the MCP tool
  schemas/prompts, the TalkBack-order algorithm, the dp math, the `host:track-devices` protocol.
- Must be rewritten in any target stack: all SwiftUI/AppKit, all `@Observable` stores, Sparkle,
  `NSWorkspace` docking, `SMAppService` login item, codesign/notarize release chain.
- macOS-only assumptions to be replaced: SDK discovery paths, `qemu-system` process-name docking,
  `CFPreferences` cross-process settings, `/data/local/tmp` temp file (device-side; fine), and
  the app-bundle-embedded MCP binary.