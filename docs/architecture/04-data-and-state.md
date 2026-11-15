# 04 — Data and state

There is **no database, no cache directory, no migrations, no serialization layer beyond JSON-RPC**.
All durable state is user preferences; all device state is re-read from the device.

## Domain models

| Model | Fields | Where | Notes |
|---|---|---|---|
| `Device` | `serial`, `model`, `state` (`device`/`unauthorized`/`offline`/`other`), `avdName`, `isBooted` | `Sources/Core/Android/Device.swift:4-33` | `isReady = isOnline && isBooted` (`:26`); `consolePort` parsed from `emulator-5554` (`:32`) |
| `DeviceInfo` | androidVersion, apiLevel, hasPlayServices, playServicesVersion, abi, screenSize, density, memory, storage (+ `summary`, `display`, `screenSizeDp`) | `Sources/Core/Android/DeviceInfo.swift:4-33` | filled from one round-trip (`:53-69`) |
| `InstalledApp` / `PreferenceEntry` | package + debuggable flag; key/type/value | `Sources/Core/Android/AppData.swift:3-17` | |
| `UINode` / `UIHierarchy` / `LayoutSnapshot` | id, class, resource-id, text, content-desc, `CGRect` bounds, 11 flags, children; density; image | `Sources/Core/Android/LayoutSnapshot.swift:4-74` | equality is by `id` only, for pointer-move performance (`:44-47`) |
| `AccessibilityItem` | order, node, role, spoken text, issues | `Sources/Core/Android/AccessibilityTraversal.swift:4-14` | derived, not stored |
| `TableRow` / `TablePage` / `SchemaEntry` / `QueryResult` | SQLite result shapes | `Sources/Core/SQLite/SQLiteSnapshot.swift:4-59` | 500-row pages (`:20`) |
| `FeatureValue` | `.toggle(Bool)` / `.number(Double)` / `.choice(String?)` | `Sources/Features/Feature.swift:64-68` | the only per-feature state |
| `Toast` | kind (info/error) + message | `Sources/State/FeatureStore.swift:5-10` | transient |
| `PanelState` | 8 exhaustive cases | `Sources/State/AppModel.swift:47-70` | the UI contract for "no usable device" |
| `UINode` cache (`HierarchyCache`) | last hierarchy per serial, so MCP `tap` can use `get_ui` refs | `Sources/MCP/Tools/ToolEnvironment.swift:74-79` | in-memory, per server process |

## Persisted state (complete list)

| Key | Domain | Type | Default | Written by | Read by | Evidence |
|---|---|---|---|---|---|---|
| `sdk.customRoot` | `UserDefaults.standard` (app) | path | absent | Settings "Choose…" | `AndroidSDK.locate` | `Sources/Core/Android/AndroidSDK.swift:11-18` |
| `panel.pinned` | standard | Bool | `true` | panel pin control | `AppModel` | `Sources/State/AppModel.swift:19`, `:22-25` |
| `panel.dockToEmulator` | standard | Bool | `true` | Settings / user drag | `PanelController` | `Sources/State/AppModel.swift:18`, `:27-30` |
| `device.lastSerial` | **`se.royan.ohmyandroid` via CFPreferences** | String | absent | `DeviceStore.selected.didSet` | `DeviceStore`, `AgentSettings.panelDevice` | `Sources/State/DeviceStore.swift:13-16`, `Sources/Core/AgentAccess.swift:43`, `:52-55` |
| `agents.access` | CFPreferences | `off`/`read`/`full` | **`full`** | Settings → AI Agents, menu | `MCPServer.callTool` | `Sources/Core/AgentAccess.swift:39`, `:44-50`, `Sources/MCP/Server/MCPServer.swift:137-144` |
| `agents.lastUsed` | CFPreferences | Date | absent | MCP server on each call | Settings | `Sources/Core/AgentAccess.swift:41`, `:58-62` |
| `agents.lastClient` | CFPreferences | String | absent | MCP server | Settings | `:40`, `:60` |
| `recording.quality` | standard | `balanced`/`best` | `balanced` | Quality feature | recording | `Sources/Features/CaptureFeatures.swift:100-118` |
| `settings.tab` | standard (`@AppStorage`) | tab id | `general` | Settings | menu → Settings | `Sources/App/SettingsView.swift:8-12` |
| Sparkle's own domain | `se.royan.ohmyandroid` | Sparkle keys | — | Sparkle | Sparkle | `Sources/App/UpdateController.swift:16-24`, `project.yml:56-61` |

Why the split: `CFPreferences` (not `UserDefaults(suiteName:)`) is used for `agents.*` and
`device.lastSerial` so the **separate MCP process** sees the same values, including a change made while
it is running (`Sources/Core/AgentAccess.swift:33-36`, `:46-50`).

The preferences domain is the app's bundle id, hard-coded twice:
`project.yml:74` and `Sources/Core/AgentAccess.swift:37`. Changing one without the other silently breaks
the access policy.

## On-disk artefacts

| Path | What | Lifetime | Evidence |
|---|---|---|---|
| `~/Desktop/Android Captures/` | Screenshots (`Screenshot <stamp>.png`), recordings (`Recording <stamp>.webm`) | user's files; never garbage-collected by the app | `Sources/Features/CaptureFeatures.swift:4-31`, `:40-46` |
| `FileManager.temporaryDirectory/ohmyandroid-db-<UUID>/` | Local copy of a device DB (+ `-wal`, `-shm`) | until Inspector closes; deleted on failure | `Sources/Core/Android/AppData.swift:61-80` |
| `FileManager.temporaryDirectory/ohmyandroid-<serial>-<UUID>.webm` | In-progress emulator recording | moved to Captures on stop (waits up to 10 s for the container to settle) | `Sources/Features/CaptureFeatures.swift:70-100` |
| `/data/local/tmp/ohmyandroid_ui.xml` **on the device** | `uiautomator dump` target, removed in the same shell line | one command | `Sources/Core/Android/LayoutSnapshot.swift:99-103` |
| `OhMyAndroid.xcodeproj`, `Sources/App/Info.plist`, `OhMyAndroid.entitlements` | Generated by XcodeGen, git-ignored | build | `project.yml:1`, `.gitignore:1-3` |
| `build/`, `dist/` | Xcode derived data + release artefacts | build | `.gitignore:5-7` |

No `~/Library/Application Support/…` directory is created. No cache. No log file — errors go to toasts
(`Sources/State/FeatureStore.swift:125-135`) or the panel's error states.

## State ownership and lifetime rules

1. **Device state is authoritative on the device.** The panel is a view with a 5 s staleness window
   (`Sources/State/FeatureStore.swift:27-37`), refreshed on hover (`Sources/UI/PanelRootView.swift:34-35`).
2. **Values are keyed by feature id and guarded by serial.** Switching device clears values and drops
   in-flight reads (`Sources/State/FeatureStore.swift:39-53`, `:72-75`).
3. **Optimistic writes.** `toggle`/`choose`/`setNumber` set the value before the command runs, then
   re-read that one feature after (`:77-111`, `:137-154`).
4. **MCP state is process-local**: device choice defaults to the panel's selection, then requires an
   explicit `device` argument if several are ready (`Sources/MCP/Tools/ToolEnvironment.swift:28-56`).
5. **UI tree refs** survive only until the next `get_ui`/`accessibility_audit` for that serial
   (`Sources/MCP/Tools/ToolEnvironment.swift:74-79`, `:90-99`).

## Serialization

- JSON-RPC only, via a hand-written `JSONValue` enum (`Sources/MCP/Protocol/JSONValue.swift`); it keeps
  integer/double distinct and encodes with `.withoutEscapingSlashes` (`Sources/MCP/Server/MCPServer.swift:169-181`).
- XML parsing via Foundation `XMLParser` for SharedPreferences and `uiautomator` dumps
  (`Sources/Core/Android/AppData.swift:94-141`, `Sources/Core/Android/LayoutSnapshot.swift:111-155`).
- `wm size` / `wm density` parsing via regex (`Sources/Core/Support.swift:18-34`).

## Evidence

`Sources/Core/Android/{Device,DeviceInfo,AppData,LayoutSnapshot}.swift`,
`Sources/Core/SQLite/SQLiteSnapshot.swift`, `Sources/Core/AgentAccess.swift`,
`Sources/State/{AppModel,FeatureStore,DeviceStore,EmulatorStore}.swift`,
`Sources/Features/CaptureFeatures.swift`, `Sources/MCP/Tools/ToolEnvironment.swift`, `.gitignore`.

## Open questions

#5 (inspector-store state), #6 (image scaling).