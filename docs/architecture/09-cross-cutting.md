# 09 — Cross-cutting concerns

## Logging

**There is no logger.** No `os_log`, no `Logger`, no file logs. Verified:
`rg -n "os_log|Logger\(|print\(" Sources` → only `print` in the MCP CLI help/version paths
(`Sources/MCP/main.swift:20-27`).

Diagnostics surface as:

| Channel | Content | Evidence |
|---|---|---|
| Toast in the panel | human-readable error text from `ShellError`/`AppError` | `Sources/State/FeatureStore.swift:125-148` |
| Panel error states | `.adbFailed(message)` shows the raw message as detail | `Sources/State/AppModel.swift:47-70`, `Sources/UI/PanelRootView.swift:50-55` |
| MCP tool result | `isError: true` + message, so the model can react | `Sources/MCP/Server/MCPServer.swift:145-152`, `Sources/MCP/Tools/Tool.swift:95-115` |

Consequence for the port: a user bug report carries no log. Adding structured diagnostics is a
*new* requirement, not a migration.

## Error taxonomy

| Type | Meaning | Produced at |
|---|---|---|
| `AppError(String)` | user-facing message already written for a human | `Sources/Core/Support.swift:4-9`; e.g. no PNG (`ADBClient.swift:23-25`), `KO:` from console (`:69-71`), empty `uiautomator` (`LayoutSnapshot.swift:108`) |
| `ShellError(command, result)` | non-zero exit; message = stderr, else stdout, else `"<cmd> failed (exit N)"` | `Sources/Core/Shell/ShellRunner.swift:13-22`, thrown at `ADBClient.swift:88-95` |
| Timeout | `AppError("<cmd> gave no answer in 60 s.")` | `ShellRunner.swift:108-111` |
| `ToolInputError(String)` | bad MCP argument; returned as a tool error so the model retries | `Sources/MCP/Tools/Tool.swift:118-125` |
| `RPCError(code, message)` | protocol-level failure (`-32700` parse, `-32601` unknown method, `-32602` unknown tool/prompt, `-32022` unsupported protocol version, `-32603` internal) | `Sources/MCP/Server/MCPServer.swift:86-93`, `:155-167` |

No error is ever retried automatically except the two cases in `05-concurrency-and-lifecycle.md`.

## Network calls, telemetry, privacy audit

| # | Destination | When | Payload | Opt-out | Evidence |
|---|---|---|---|---|---|
| 1 | `https://github.com/ateymoori/oh-my-android/releases/latest/download/appcast.xml` | Sparkle scheduled check, ≤ once/day; or user-initiated check | standard HTTP GET; no device data; standard User-Agent | Settings → Updates ("check automatically") | `project.yml:57-61`, `Sources/App/UpdateController.swift:16-30`, `:21-24` |

That is the complete list. No analytics, no crash reporting, no fonts, no CDNs, no `URLSession` outside
Sparkle: `rg -n "URLSession|http" Sources` → hits only for `SUFeedURL` docs, the Android Studio link,
and the project URL strings.

Data that never leaves the machine: device serials, screenshots, UI trees, SharedPreferences, SQLite
copies, captured PNG/WebM. Screenshots and recordings land in `~/Desktop/Android Captures`
(`Sources/Features/CaptureFeatures.swift:5`) — a user-visible location, by design.

Redaction: **none**. Serials appear in device pickers and in MCP error text
(`Sources/MCP/Tools/ToolEnvironment.swift:34-56`), and `device.lastSerial` is written to preferences
(`Sources/State/DeviceStore.swift:13-15`). Acceptable for a local dev tool; a Windows port that adds any
telemetry would need explicit redaction rules.

## Auto-update

| Aspect | Behaviour | Evidence |
|---|---|---|
| Framework | Sparkle 2.10.0, pinned `exactVersion` | `project.yml:26-31` |
| Feed | GitHub release asset of the latest release | `project.yml:57` |
| Trust | EdDSA public key pinned in Info.plist + same Developer ID required; `SURequireSignedFeed`, `SUVerifyUpdateBeforeExtraction` | `project.yml:58-61` |
| UX | never interrupts: badge dot on the menu-bar icon + "Update to X…" menu item | `Sources/App/UpdateController.swift:34-52`, `Sources/App/OhMyAndroidApp.swift:21-24` |
| Scheduling | daily (`SUScheduledCheckInterval: 86400`), `SUAutomaticallyUpdate: false` | `project.yml:60-61` |
| Release pipeline | separate "update" zip so the download counter only counts installs | `Scripts/release.sh:72-85` |

## Single instance / activation

- No lock file, no `NSRunningApplication` check, no `SMLoginItem` handshake. LaunchServices normally
  prevents two copies of the same bundle; a copied `.app` elsewhere would run twice and both would talk
  to the same adb server (harmless, but two panels).
- `applicationShouldTerminateAfterLastWindowClosed` → `false`; reopening the app shows the panel
  (`Sources/App/AppDelegate.swift:59-64`).
- The panel is a **non-activating** panel when pinned, so clicking a feature does not steal focus from
  the app under test (`Sources/Windows/FloatingPanel.swift:22-40`). This is the single most important
  UX property to preserve on Windows (needs `WS_EX_NOACTIVATE`).

## Permissions / elevation

| Thing | Requirement | Evidence |
|---|---|---|
| Running `adb` | none (user-owned SDK binary) | `Sources/Core/Android/AndroidSDK.swift:34-36` |
| USB device access | macOS needs no driver; Windows needs OEM/WinUSB drivers → a **new** first-run problem | `UNVERIFIED` (not in this repo) |
| Reading other apps' windows (docking) | avoided by polling a 1 Hz timer, explicitly because Accessibility permission would otherwise be required | `Sources/Windows/PanelController.swift:3-6` |
| App Sandbox | **disabled** — required to spawn the SDK's binaries and read the Desktop | `project.yml:69-72` |
| Hardened runtime | enabled (needed for notarization) | `project.yml:16` |
| Elevation | never requested | — |
| Screen recording permission (macOS) | not needed: it screenshots the *device*, not the Mac | `Sources/Core/Android/ADBClient.swift:21-27` |

## Security review (author's posture, verified against code)

| Area | Finding | Evidence |
|---|---|---|
| Shell injection | user text is single-quoted for the device `sh`; the APK path is argv, not shell | `Sources/Core/Support.swift:14`, `Sources/Features/AppFeatures.swift:49` |
| Package-name injection (MCP) | regex-validated before use | `Sources/MCP/Tools/ToolEnvironment.swift:101-108` |
| Untrusted device output | parsed only; the one value that feeds another command (`resolve-activity` → `am start -n`) is quoted | `Sources/MCP/Tools/AppTools.swift:48-50` |
| **Path traversal from device filenames (potential)** | `pullDatabase(name:)` writes to `directory.appending(path: name)` where `name` comes from the device's `ls -1 databases` (`AppData.swift:55-60`, `:61-70`). `appending(path:)` does not normalize `..`. A hostile/compromised device could report `../../..` names and get a file written outside the temp dir. **Not confirmed as reachable** — the caller (`InspectorStore`) was not read. | `Sources/Core/Android/AppData.swift:55-70` |
| Private app data | only via `run-as`, i.e. `debuggable=true` builds; read-only; local copies deleted on failure | `Sources/Core/Android/AppData.swift:19-34`, `:61-80` |
| SQL | read-only enforced twice (statement check + `PRAGMA query_only`), `ATTACH` disabled | `Sources/Core/SQLite/SQLiteSnapshot.swift:63-69`, `:86`, `:102-107` |
| Agent capability gating | every tool declares `effect`; `read` → needs `readOnly`, others → `full`; refusal message tells the user where to change it | `Sources/MCP/Tools/Tool.swift:6-14`, `Sources/MCP/Server/MCPServer.swift:137-144` |
| Agent default | **full control** when the app has never set a value | `Sources/Core/AgentAccess.swift:44` |
| Update integrity | EdDSA-signed feed + signed zip + Developer ID equality | `project.yml:56-61` |
| Notarization hygiene | refuses to package if `get-task-allow` is present or the signature lacks a timestamp | `Scripts/release.sh:49-58` |
| Secrets | none in the repo | `rg -ni "token\|secret\|api_key"` → only EdDSA *public* key |

## Packaging / signing / native deps

Zip + notarization + staple, Homebrew cask, Sparkle appcast, and an `.mcpb` of the server alone
(`Scripts/release.sh:61-120`). No native third-party libraries: the only binary dependency is
`Sparkle.framework`, embedded and signed inside-out (`Scripts/release.sh:33-44`). SQLite comes from the
system (`import SQLite3`, `Sources/Core/SQLite/SQLiteSnapshot.swift:2`).

## Evidence

`Sources/Core/**`, `Sources/MCP/**`, `Sources/App/UpdateController.swift`, `project.yml`,
`Scripts/release.sh`, `SECURITY.md`.

## Open questions

#4 (whether the traversal in `pullDatabase` is reachable), #1 (Windows default agent access).