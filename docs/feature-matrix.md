# Feature matrix — port backlog

Every user-visible capability found in the code, with an MVP tier. Complexity `S` <1 h, `M` ~1 day,
`L` several days, `XL` a week+ (estimate for a competent dev in a fresh Windows codebase, not for the
Swift original).

Tier meaning: **P0** = without it the Windows app is not a usable equivalent of upstream;
**P1** = needed for parity, can ship one release later; **P2** = nice to have;
**NG** = non-goal for the fork (explicitly dropped, with reason).

## Device connection and shell

| Feature | User value | Source impl | Cx | Win feasibility | Deps | Tier | Notes |
|---|---|---|---|---|---|---|---|
| Find the Android SDK + `adb` | nothing works without it | `Sources/Core/Android/AndroidSDK.swift:13-36` | S | High | — | P0 | Windows search order is different; keep "choose folder" + persist |
| Start / restart the adb server | fixes stuck devices | `Sources/Core/Android/ADBClient.swift:43-47`, `:82-86` | S | High | job objects, `CREATE_NO_WINDOW` | P0 | **Implemented (MVP)**: `adb_start`/`adb_stop`, all stdio null, behind a confirm dialog (`crates/adbkit/src/runner.rs`) |
| List devices (state, model, boot, AVD name) | the core list | `Sources/Core/Android/ADBClient.swift:49-121` | M | High | — | P0 | **Implemented (MVP, refresh-only)** for serial/model/state; boot + AVD name not yet (`crates/adbkit/src/parser.rs`) |
| Live hotplug updates | no refresh button needed | `Sources/Core/Android/DeviceTracker.swift:96-118` | M | Medium | winsock | P0 | Raw `host:track-devices`; fallback = 2 s polling |
| Remember last device | convenience | `Sources/State/DeviceStore.swift:13-16`, `:57-63` | S | High | — | P0 | |
| Boot/recovery polling | panel works when boot finishes | `Sources/State/DeviceStore.swift:75-90` | S | High | — | P0 | 4 s while not ready |
| Device summary (Android ver, API, Play, ABI, display, RAM, storage) | orientation | `Sources/Core/Android/DeviceInfo.swift:41-69` | M | High | — | P0 | One batched shell |
| The 8-state panel UI | every edge case has a screen | `Sources/State/AppModel.swift:47-70`, `Sources/UI/PanelRootView.swift:43-...` | M | High | — | P0 | Best idea in the repo — replicate exactly |
| List AVDs / start an AVD | works with Studio closed | `Sources/Core/Android/EmulatorLauncher.swift:8-27`, `Sources/State/EmulatorStore.swift:26-46` | M | High | `emulator.exe` | P1 | 180 s start limit |
| Dock the panel beside the emulator window | the product's signature UX | `Sources/Windows/PanelController.swift:97-111` | L | **Low** | Win32 window enumeration, no-activate window | P2 | Windows emulator window class/title is not stable; needs a spike |
| Non-activating top-most panel | click without stealing focus | `Sources/Windows/FloatingPanel.swift:22-40` | M | Medium | `WS_EX_NOACTIVATE`, `WS_EX_TOPMOST` | P0 | Without this the app is annoying to use |
| Menu-bar / tray icon + menu | entry point | `Sources/App/OhMyAndroidApp.swift:8-55` | S | High | `NotifyIcon` | P0 | |
| Open at login | convenience | `Sources/App/SettingsView.swift:29` | S | High | registry Run key / Task Scheduler | P1 | |
| In-app updates | keep users current | `Sources/App/UpdateController.swift`, `Scripts/release.sh:72-85` | M | Medium | Velopack/NetSparkle + own key | P2 | Do **not** reuse upstream feed/keys |

## Appearance (the reason people install it)

| Feature | Source | Cx | Tier | Notes |
|---|---|---|---|---|
| Dark mode | `Sources/Features/DisplayFeatures.swift:3-14` | S | P0 | `cmd uimode night` |
| Font size stepper (7 steps) | `:16-30` | S | P0 | `settings put system font_scale` |
| Display size stepper (7 steps) | `:32-61` | S | P0 | `wm density` ratio vs physical; `reset` at 100 % |
| Language / locale picker (12 incl. RTL + 2 pseudo-locales) | `:63-100` | M | P0 | `cmd locale set-device-locale` with `am update-config` fallback |
| Rotate device | `FeatureCatalog.swift:11` via `EmulatorFeatures` | S | P1 | emulator console |
| Bold text | `Sources/Features/AccessibilityFeatures.swift:27-38` | S | P1 | `secure font_weight_adjustment` |
| Invert colors | `:40-51` | S | P1 | |
| TalkBack on/off | `:3-25` | S | P0 | two `settings put` + `delete`; the headlining accessibility feature |

## Layout & measurement

| Feature | Source | Cx | Tier | Notes |
|---|---|---|---|---|
| Layout Inspector window (frozen screenshot) | `Sources/UI/Layout/LayoutInspectorView.swift`, `Sources/State/LayoutInspectorStore.swift` | XL | P1 | Custom canvas; the biggest UI work item |
| dp hover measurements (size, gaps, padding) | same | L | P1 | `pixels × 160 / density`; pure maths is reusable |
| Design overlay (drop a Figma export, compare) | `Sources/App/AppDelegate.swift:48-57`, `LayoutInspectorView.swift:140-151` | L | P1 | Image + alpha; portable |
| TalkBack order audit + issues (no label, NAF, <48 dp, undescribed image) | `Sources/Core/Android/AccessibilityTraversal.swift:19-115` | M | P0 | **Pure logic, no UI deps, highest reuse value** |
| UI tree read (`uiautomator dump`) | `Sources/Core/Android/LayoutSnapshot.swift:82-109` | M | P0 | Retry once on animation |
| Bounds overlay / overdraw / GPU bars | `Sources/Features/DebugFeatures.swift:34-38`, `GenericFeatures.swift:22-36` | S | P1 | `setprop` + `service call activity 1599295570` (`UNVERIFIED` stability) |
| Show taps / pointer location | `DebugFeatures.swift:40-43` | S | P1 | |
| Animations off | `DebugFeatures.swift:4-18` | S | P0 | Batch the 3 puts into one shell |
| Dev options shortcut | `DebugFeatures.swift:45-57` | S | P1 | |
| Snapshots resolve/save/load (emulator) | `Sources/MCP/Tools/EmulatorTools.swift:31-32` | S | P2 | console |

## Capture

| Feature | Source | Cx | Tier | Notes |
|---|---|---|---|---|
| Screenshot → PNG on Desktop + clipboard | `Sources/Features/CaptureFeatures.swift:33-46`, `ADBClient.swift:21-27` | S | P0 | Strip the foldable warning prefix before the PNG magic |
| Open captures folder | `:166-176` | S | P0 | |
| Screen record (emulator console, WebM, ≤3 min) | `:52-99`, `:126-138` | M | P2 | Console only; arg-splitting trap |
| Recording quality choice | `:142-164` | S | P2 | |
| Live screen mirroring | — | — | **NG / new feature** | Not in upstream. See `docs/architecture/07-mirroring-and-control.md` |

## App actions (foreground-app aware)

| Feature | Source | Cx | Tier | Notes |
|---|---|---|---|---|
| Read the foreground package | `Sources/Core/Android/ForegroundAppReader.swift:9-14` | S | P0 | `dumpsys activity activities \| grep -m1 Resumed` |
| Restart app | `Sources/Features/AppFeatures.swift:82-85` | S | P0 | force-stop + monkey |
| Force stop / Clear data / Uninstall / Reset permissions | `:73-97` | S | P0 | destructive → confirm |
| App info screen | `:86-89` | S | P1 | |
| Deep link / open URL | `:3-18` | S | P1 | quote the URL for the device shell |
| Install APK (picker + drag-drop) | `Sources/Features/AppFeatures.swift:34-53`, `Sources/UI/FileDrop.swift` | M | P0 | argv, not shell; no progress bar upstream — add one |
| Package list + debuggable detection | `Sources/Core/Android/AppData.swift:36-44` | M | P1 | one `for` loop shell over `pm list packages -3` |

## App data

| Feature | Source | Cx | Tier | Notes |
|---|---|---|---|---|
| SharedPreferences browser | `Sources/Core/Android/AppData.swift:50-53`, `:94-141`, `UI/Inspector/InspectorView.swift:55-84` | M | P2 | XML parse + typed rows |
| SQLite browser (tables, 500-row pages, schema) | `Sources/Core/SQLite/SQLiteSnapshot.swift:19-52` | M | P2 | needs a SQLite provider on Windows |
| Read-only ad-hoc SQL | `SQLiteSnapshot.swift:63-69` | M | P2 | `query_only` + no `ATTACH` + single statement |

## Network / hardware simulation (emulator console)

| Feature | Source | Cx | Tier | Notes |
|---|---|---|---|---|
| Network profile (speed/delay) | `Sources/Features/NetworkFeatures.swift:28-36` | S | P2 | `adb emu network` |
| Airplane mode / Wi-Fi / mobile data | `:50-76` | S | P1 | `cmd connectivity`, `svc wifi`, `svc data` — real devices too |
| Location presets + custom | `Sources/Features/LocationFeatures.swift:32-48` | S | P2 | `adb emu geo fix <lon> <lat>` |
| Battery level / charging / AC | `Sources/Features/HardwareFeatures.swift:12-34` | S | P2 | console |
| Incoming SMS / call, fingerprint | `:47`, `FeatureCatalog.swift:62-64` | S | P2 | console; SMS text must not contain spaces (`UNVERIFIED`) |

## Navigation and input

| Feature | Source | Cx | Tier | Notes |
|---|---|---|---|---|
| Back / Home / Recents / Hide keyboard | `Sources/Features/InputFeatures.swift:4-23` | S | P0 | `input keyevent` |
| Type text (ASCII only) | `:25-43` | S | P0 | keep the ASCII guard and the `%s` space rule |
| Soft keyboard toggle | `:45-49` | S | P1 | |
| Tap by ref / text / dp, swipe, press key | `Sources/MCP/Tools/InputTools.swift:8-174` | M | P1 | MCP-only today; could become panel features |

## MCP server (agent surface)

| Feature | Source | Cx | Tier | Notes |
|---|---|---|---|---|
| stdio JSON-RPC, dual-era protocol | `Sources/MCP/main.swift`, `Server/MCPServer.swift:7-181` | L | P1 | Protocol is portable; `Scripts/test-mcp.py` is the acceptance test |
| 18 tools (device, screen, input, app, data, emulator) | `Sources/MCP/Tools/*.swift`, `Server/Catalog.swift:5-8` | L | P1 | Schemas/prompts reusable verbatim |
| 3 prompts (accessibility review, UI matrix, debug crash) | `Server/Catalog.swift:18-60` | S | P1 | Pure text |
| Access policy Off/Read/Full | `Sources/Core/AgentAccess.swift:4-67` | M | P1 | Needs a Windows settings store; **default deliberately** |
| Compact UI-tree text format (dp, refs) | `Sources/MCP/Format/UITreeText.swift` | M | P1 | Keep byte-compatible for prompt stability |
| `.mcpb` bundle + registry metadata | `mcpb/manifest.json`, `server.json`, `glama.json`, `.claude-plugin/*` | S | P2 | add `win32` platform |
| Claude Code skill (`skills/android-ui-check`) | `skills/android-ui-check/SKILL.md` | S | P2 | advisory workflow |

## Host integration

| Feature | Source | Cx | Tier | Notes |
|---|---|---|---|---|
| Settings window (SDK, captures path, dock, login) | `Sources/App/SettingsView.swift` | M | P0 | |
| Toasts (info/error) | `Sources/UI/ToastView.swift`, `State/FeatureStore.swift:125-135` | S | P0 | |
| File picker for APK / design image | `Sources/App/AppDelegate.swift:20-27` | S | P0 | `IFileOpenDialog` |
| Drag-and-drop APK onto the panel | `Sources/UI/PanelRootView.swift:30`, `FileDrop.swift` | M | P1 | OLE drop target |
| "Open with" a PNG/JPEG → overlay | `Sources/App/AppDelegate.swift:48-57`, `project.yml:63-67` | M | P2 | file-association registration |
| Hide/show panel from tray, ⌘, settings, ⌘Q | `Sources/App/OhMyAndroidApp.swift:37-55` | S | P0 | |
| Sparkle-style silent updates | `Sources/App/UpdateController.swift` | M | P2 | |
| Non-English UI | — | — | NG | Upstream is English-only; add later deliberately |
| Telemetry / crash reporting | — | — | NG | Violates the project's stated posture (`SECURITY.md:9-16`) |

## Explicit non-goals derived from absent code

File push/pull or a device file browser · wireless debugging (`tcpip`/`pair`/`connect`) · root/remount ·
reboot · logcat streaming with filters · `screencap` video streaming · multi-device side-by-side control ·
plugin system · themes.

## Suggested MVP cut (P0 only)

SDK discovery → adb lifecycle → device list + hotplug + the 8 panel states → device summary →
screenshot + open captures → install APK (picker + drop) → dark mode / font size / display size /
locale / TalkBack / animations off → back/home/recents/type → foreground-app actions (restart, clear
data, force stop, uninstall, reset permissions) → toast + busy + confirm for destructive actions →
UI tree read + TalkBack audit (logic + list UI).

That is upstream minus: Layout Inspector canvas, Design overlay, app-data inspectors, emulator console
simulation (network/location/battery/SMS/snapshots/recording), AVD start, docking, updates, and the MCP
server.

## Evidence

File references in each row. Feature inventory cross-checked against
`Sources/Features/FeatureCatalog.swift:5-71` and `Sources/MCP/Server/Catalog.swift:5-8`.