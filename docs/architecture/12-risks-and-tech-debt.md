# 12 — Risks and technical debt (ranked by impact on a Windows port)

| # | Risk | Impact on the port | Evidence | Mitigation |
|---|---|---|---|---|
| R1 | **The entire UI is SwiftUI on a macOS-26-only API surface.** `glassEffect`, `WindowDragGesture`, `NSPanel` levels, `MenuBarExtra`, `SMAppService`, `NSWorkspace` docking. | 100 % of the presentation layer and ~40 % of the code is unportable. This is a rewrite, not a port. | `project.yml:11`, `Sources/UI/PanelRootView.swift:20-22`, `Sources/Windows/FloatingPanel.swift:6-47`, `Sources/Windows/PanelController.swift:97-111`, `Sources/App/SettingsView.swift:29` | Pick the target stack on evidence (ADR-0001 at Gate 2) and treat `Sources/UI/**`, `Sources/Windows/**`, `Sources/State/**` as **reference-only** |
| R2 | **No automated tests beyond the MCP protocol.** | Any rewrite has no safety net; parser regressions will be invisible. | `Scripts/test-mcp.py` is the only test; `rg XCTest` → 0 | Build a fake-`adb` fixture harness **first** (M0), then port parsers against it |
| R3 | **Device-output parsing is implicit and version-fragile**: `adb devices -l` text, `wm size` regex, `uiautomator` XML, `df -k /data`, `dumpsys package` grep, `logcat -v tag`. | Silently degrades on Android 16+ / OEM shells; must be re-verified, not assumed. | `Sources/Core/Android/ADBClient.swift:98-104`, `Sources/Core/Support.swift:18-34`, `Sources/Core/Android/DeviceInfo.swift:41-51` | Golden-file tests per command from real transcripts; explicit `PARSE_FAILED` states |
| R4 | **No logging at all.** Errors only become toasts. | Supporting Windows users will be guesswork; also blocks telemetry-free diagnostics. | `docs/architecture/09-cross-cutting.md` | Add a rolling local log + "copy diagnostics" action (no network) |
| R5 | **Access policy defaults to "full control"** and is stored in a macOS preferences domain read cross-process. | A Windows port must invent an equivalent store (registry/key file) and must decide the default deliberately — copying `full` silently is a security regression. | `Sources/Core/AgentAccess.swift:44`, `:33-50`, `Sources/MCP/Server/MCPServer.swift:137-144` | Explicit product decision + first-run prompt; default `readOnly` |
| R6 | **Classic Mac assumptions in path/binary handling**: `adb` has no `.exe` suffix, SDK roots are `/opt/homebrew/…`, `~/Library/Android/sdk`, `~/Desktop/Android Captures`, `emulator/emulator`. | Naive copy produces an app that finds nothing on Windows. | `Sources/Core/Android/AndroidSDK.swift:9`, `:13-32`, `Sources/Core/Android/EmulatorLauncher.swift:7`, `Sources/Features/CaptureFeatures.swift:5` | A `PlatformPaths` abstraction (`%LOCALAPPDATA%\Android\Sdk`, `adb.exe`, `%USERPROFILE%\Desktop\Android Captures`) |
| R7 | **Raw POSIX socket + `SO_NOSIGPIPE`/`Darwin.connect`** for hotplug. | Not compilable on Windows; and Windows has no `SIGPIPE` concept, so a naive port crashes instead of erroring. | `Sources/Core/Android/DeviceTracker.swift:2`, `:96-118` | Winsock wrapper (`WSAStartup`, `SO_NOSIGPIPE` equivalent unnecessary, handle `WSAECONNRESET`) |
| R8 | **Unbounded process fan-out** (one `adb` per device per refresh, one per non-batchable feature). | On Windows, process creation is more expensive; this could make the panel sluggish. | `Sources/Core/Android/ADBClient.swift:53-61`, `Sources/State/FeatureStore.swift:59-70` | Concurrency cap + reuse of the batched-shell trick |
| R9 | **Emulator-only features are console-protocol based** (`adb emu`), and console args are split on spaces. | Behaviour differs per emulator version; SMS/location/battery may fail silently. | `Sources/Features/CaptureFeatures.swift:52-58`, `Sources/Features/HardwareFeatures.swift:12-47` | Treat as P1; keep errors surfaced per feature |
| R10 | **The MCP server is compiled into the app bundle and shares all app code.** | Splitting it for Windows (single binary vs sibling exe) is an architectural decision that affects packaging, signing and the `.mcpb`/registry metadata. | `project.yml:83-93`, `mcpb/manifest.json`, `server.json` | Decide at Gate 2 (`docs/open-questions.md` #1) |
| R11 | **Release/distribution is Apple-specific end-to-end** (codesign inside-out, notarytool, staple, Homebrew cask, Sparkle appcast). | Nothing is reusable except the *sequence* of checks. Signing + updater + installer must be re-chosen. | `Scripts/release.sh:23-120` | New pipeline: portable zip or MSIX + a Windows updater with its own key |
| R12 | **Path traversal in `pullDatabase(name:)`** (device-supplied filename appended without normalization). | Small security hole inherited by any port that copies the logic. | `Sources/Core/Android/AppData.swift:55-70` | Sanitize to a basename; reject separators and `..`; confirm reachability (open question #4) |
| R13 | **macOS-only naming/metadata** (`se.royan.*`, "Royan AB", `ateymoori` feed, Homebrew tap) baked into prefs domain, feed URL, manifest files. | A fork that keeps these will half-update or mis-attribute. | `project.yml:57-58`, `:74`, `Sources/Core/AgentAccess.swift:37`, `mcpb/manifest.json:8-17` | Fork identity pass (this repo's Phase 1 work) |
| R14 | **`project.yml` is the only build truth**, and the `.xcodeproj` is generated. | Any new target (e.g. a Windows solution) must not drift from it; docs must name both. | `project.yml:1`, `.gitignore:1-2` | Keep `project.yml` for the reference build; add a separate target tree for Windows |

## Non-risks (checked, benign)

- No telemetry to remove (only the update check).
- No database migrations to port.
- No bundled binaries to relicense.
- No adb library to swap (only argv + one socket).
- No background daemon to re-implement.
- No mirroring/video stack to port (it does not exist).

## Technical debt worth fixing in the fork, regardless of stack

1. Add a logger (R4) and a "copy diagnostics" action.
2. Add fixture tests for every parser (R3) before touching anything.
3. Normalize `pullDatabase` filenames (R12).
4. Replace the three sequential `settings put` spawns in `AnimationsOffFeature` with one batched shell
   (`Sources/Features/DebugFeatures.swift:15-17`).
5. Give the MCP server its own module boundary so "server-only" builds are possible (R10).
6. Document the `DebugPropertyToggleFeature`'s magic `service call activity 1599295570`
   (`Sources/Features/Feature.swift:50-52`) — it is the internal `SET_ACTIVITY_WATCHER`-ish transaction
   id; `UNVERIFIED` whether it is stable across Android versions.

## Evidence

All rows cite files above. Cross-references: `docs/architecture/00-overview.md`,
`05-concurrency-and-lifecycle.md`, `06-adb-integration.md`, `09-cross-cutting.md`,
`10-dependencies-and-licenses.md`, `11-build-run-test.md`.