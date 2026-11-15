# 04 — Feature parity plan

Tiers per `../feature-matrix.md`; milestones per `18-plan-and-backlog.md`. `M?` = first milestone that
ships it. **Excluded by decision:** MCP server, agent access policy, `.mcpb` bundle, Claude skill.

## 1. Ship list

| Feature | Upstream ref | Tier | M | Windows approach |
|---|---|---|---|---|
| SDK discovery + validation | `AndroidSDK.swift:13-36` | P0 | M0 | `PlatformToolsLocator`, registry + `LOCALAPPDATA` + manual picker |
| adb server start/restart | `ADBClient.swift:43-47`, `:82-86` | P0 | M0 | `AdbRunner`, explicit user action only |
| Device list + states | `ADBClient.swift:49-121` | P0 | M0 | `devices -l` parse + batched `getprop` |
| Hotplug (push) | `DeviceTracker.swift:96-118` | P0 | M0 | raw `host:track-devices` + 2 s poll fallback |
| Remember last device / boot polling | `DeviceStore.swift:13-90` | P0 | M0 | settings + 4 s recovery poll |
| Panel with 8 states | `AppModel.swift:47-70` | P0 | M0 | `windows/panel` + pure `panelState()` |
| Tray icon + menu | `OhMyAndroidApp.swift:8-55` | P0 | M0 | `TrayIcon` config + Rust menu |
| Non-activating top-most panel | `FloatingPanel.swift:22-40` | P0 | M0 | `WS_EX_NOACTIVATE`+`WS_EX_TOPMOST` via `windows` crate |
| Device summary | `DeviceInfo.swift:41-69` | P0 | M1 | one batched shell |
| Appearance: dark/font/display/locale/TalkBack/animations | `DisplayFeatures.swift`, `AccessibilityFeatures.swift`, `DebugFeatures.swift:4-18` | P0 | M1 | feature catalog → `settings put`/`cmd` |
| Screenshot + captures folder | `CaptureFeatures.swift:33-46`, `:166-176` | P0 | M1 | `exec-out screencap` → file + clipboard |
| APK install (picker + drag-drop) | `AppFeatures.swift:34-53`, `FileDrop.swift` | P0 | M2 | dialog + streamed install with progress (new) |
| App actions (restart/stop/clear/uninstall/reset-perms/info) | `AppFeatures.swift:73-97` | P0 | M2 | confirm + serialized runner |
| Foreground app read | `ForegroundAppReader.swift:9-14` | P0 | M2 | `dumpsys activity activities` |
| Input: back/home/recents/type/keyboard | `InputFeatures.swift:4-49` | P0 | M1 | `input keyevent` / `input text` (ASCII guard) |
| Toasts, busy, confirm | `ToastView.swift`, `FeatureStore.swift:125-135` | P0 | M0 | shared components |
| Settings window | `SettingsView.swift` | P0 | M1 | Svelte settings + Rust `SettingsStore` |
| Logcat viewer | — (new) | P1 | M1 | biggest new feature; coalesced channel |
| File browser (ls/push/pull/mkdir/delete) | — (new) | P1 | M2 | `fs_*` + progress |
| Wireless debugging (pair/connect) | — (new) | P1 | M4 | `adb pair`/`adb connect` + mDNS `UNVERIFIED` |
| UI tree dump + TalkBack audit | `AccessibilityTraversal.swift:19-115`, `LayoutSnapshot.swift:82-109` | P0 | M2 | logic is pure, highest reuse value |
| AVD list + start | `EmulatorLauncher.swift:8-27` | P1 | M4 | `emulator.exe` |
| Network/location/battery/SMS simulation | `NetworkFeatures.swift`, `LocationFeatures.swift`, `HardwareFeatures.swift` | P2 | M4 | `adb emu` console |
| Screen record | `CaptureFeatures.swift:52-99` | P2 | M4 | console, WebM |
| SharedPreferences + SQLite browser | `AppData.swift`, `SQLiteSnapshot.swift` | P2 | M5 | `rusqlite` (bundled) |
| Layout Inspector (dp overlay, design compare) | `UI/Layout/*` | P1 | M5 | canvas; largest UI item |
| Mirroring | — (new) | — | M3 | ADR-0002; spike-gated |
| Auto-update | `UpdateController.swift` | P2 | M6 | `tauri-plugin-updater`, new keys |
| Start at login | `SettingsView.swift:29` | P1 | M5 | Task Scheduler / Run key |
| Dock panel to emulator window | `PanelController.swift:97-111` | P2 | post-1.0 | Win32 HWND tracking; emulator title not stable |

## 2. Deliberate omissions vs upstream

| Upstream | Why dropped |
|---|---|
| `Sources/MCP/**` (18 tools, 3 prompts, stdio server) | D3: app only |
| `Sources/Core/AgentAccess.swift` (Off/Read/Full policy) | only existed to gate the MCP server |
| `mcpb/manifest.json`, `server.json`, `glama.json`, `.claude-plugin/*`, `skills/android-ui-check` | registry/agent packaging |
| `Scripts/test-mcp.py`, `.github/workflows/build.yml` MCP step | macOS-only; workflow untouched anyway (D4) |
| Sparkle | replaced by `tauri-plugin-updater` |

## 3. Behavioural parity notes (do not "improve" silently)

| Behaviour | Upstream | Port decision |
|---|---|---|
| One adb process per batched read | `ADBClient.swift:31-36` | keep |
| 60 s default timeout | `ShellRunner.swift:32-113` | keep; 600 s for install/pull |
| ASCII-only text injection | `InputFeatures.swift:25-43` | keep + explain in UI |
| Emulator console arg splitting on spaces | `CaptureFeatures.swift:52-58` | keep quoting rules; test them |
| `agents.access` default **full** | `AgentAccess.swift:44` | **not applicable** (no MCP); nothing to inherit |
| Silently ignoring foldable warning prefix before PNG | `ADBClient.swift:21-27` | keep |
| No logging | — | **improve**: we add structured logs (`15-plugin-and-security.md` §8) |
| No tests | — | **improve**: fixtures + fake-adb (`17-testing-strategy.md`) |

## 4. Parity acceptance test (v1.0)

For the P0 list above: on a clean Windows 11 machine with a physical device *and* an emulator, every
upstream P0 behaviour is reachable in ≤2 clicks from the panel, with an explicit state for every failure
mode in `docs/architecture/08-ui-ux-map.md` §3. Signed off in `M6` by a written checklist in
`docs/windows-port/parity-checklist.md` (created in M5).