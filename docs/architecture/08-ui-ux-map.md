# 08 — UI/UX map

SwiftUI with a small AppKit shell. No i18n (hard-coded English, `SWIFT_EMIT_LOC_STRINGS: NO`,
`project.yml:79`), no custom theme system beyond `Theme` tokens, no VoiceOver-specific labels except
where macOS supplies them (one explicit case: the menu-bar icon description,
`Sources/App/UpdateController.swift:76`).

## Window inventory

| Surface | Size / style | Purpose | Evidence |
|---|---|---|---|
| Menu-bar item | icon, orange dot when an update is pending | entry point: show/hide panel, inspectors, restart adb, AI access, updates, quit | `Sources/App/OhMyAndroidApp.swift:8-55`, `Sources/App/UpdateController.swift:56-78` |
| Floating panel | 236 × 700, borderless, glass, corner 26, padding 12 | the product | `Sources/UI/Theme.swift:5-14`, `Sources/UI/PanelRootView.swift:5-30`, `Sources/Windows/FloatingPanel.swift:6-47` |
| Data Inspector | `ToolWindow` | SharedPreferences / SQLite of a debuggable app | `Sources/App/AppDelegate.swift:13-16`, `Sources/UI/Inspector/InspectorView.swift:4-133` |
| Layout Inspector | `ToolWindow` | frozen screenshot + dp measurements + design overlay + TalkBack audit | `Sources/App/AppDelegate.swift:14-16`, `Sources/UI/Layout/LayoutInspectorView.swift:7-319` |
| Settings | SwiftUI `Settings` scene, 540 pt wide, 2 tabs | SDK path, dock, login item, captures path, updates, agent access | `Sources/App/SettingsView.swift:8-25` |
| Toasts | overlay at the bottom of the panel | transient info/error | `Sources/UI/ToastView.swift`, `Sources/UI/PanelRootView.swift:29` |

## Panel component tree

```
PanelRootView (Sources/UI/PanelRootView.swift:3)
├─ HeaderView (Sources/UI/HeaderView.swift:4)            device picker? pin, restart adb, settings
├─ DeviceInfoView (Sources/UI/DeviceInfoView.swift:4)    Android version, API, Play, display, memory
├─ content: switch on AppModel.PanelState
│   ├─ .ready → FeatureGridView (Sources/UI/FeatureGridView.swift:3)
│   │            └─ 3 columns × FeatureCell (Sources/UI/FeatureCell.swift:4)
│   │                 ├─ StepperPopover (:133)
│   │                 └─ TextActionPopover (:155)
│   └─ .sdkMissing / .adbFailed / .starting / .booting / .unauthorized / .offline / .noDevice
│        → PanelMessage (Sources/UI/PanelMessage.swift:5) + PanelMessageButton (:58)
└─ ToastView overlay (:29)
```

Panel state coverage (this is the strongest UI idea in the repo and should be preserved in any port):

| `PanelState` | What the user sees | Evidence |
|---|---|---|
| `.sdkMissing` | "Android SDK Not Found", Choose SDK…, link to Android Studio | `Sources/UI/PanelRootView.swift:44-49` |
| `.adbFailed(msg)` | "adb Not Responding" + Restart adb, with the error as detail | `:50-55` |
| `.starting(avd)` | "Starting <AVD>…" while an AVD launched from the panel boots | `:56-58` |
| `.booting(device)` | "Starting <device>", waits for `sys.boot_completed` | `:59-62` |
| `.unauthorized(device)` | "accept USB debugging on the phone" | `Sources/Core/Android/Device.swift:10-16` |
| `.offline(device)` | reconnect / restart adb hints | `Sources/UI/PanelRootView.swift:66-71` |
| `.noDevice` | list/add an emulator | `:72-...` |
| `.ready(context)` | the feature grid, live values | `:43-44` |

## Feature grid behaviour

| Aspect | Behaviour | Evidence |
|---|---|---|
| Layout | fixed 3 columns, 46 pt buttons, 64 pt cells, 236 pt panel | `Sources/UI/Theme.swift:14-16` |
| Ordering | category order = declaration order; within a category = catalog order; emulator-only hidden on real devices | `Sources/Features/FeatureCatalog.swift:3`, `:79-85` |
| Live value | icon reflects device state (on/off, step index, selected option) | `Sources/State/FeatureStore.swift:29`, `Sources/UI/FeatureCell.swift` |
| Busy state | per-feature spinner; a second tap is dropped rather than queued | `Sources/State/FeatureStore.swift:143` |
| Optimistic update | value flips before the command, corrected by the re-read | `:77-111` |
| Errors | toast, error toasts live 5 s and are announced to VoiceOver | `:125-135` |
| Refresh trigger | **hover** into the panel, if the last full read is >5 s old | `Sources/UI/PanelRootView.swift:34-35`, `Sources/State/FeatureStore.swift:27-37` |
| Destructive actions | flagged `isDestructive`; panel asks first | `Sources/Features/Feature.swift:158-163` (flag), `CONTRIBUTING.md:30-31` (behaviour claim) — the confirm UI itself is in `FeatureCell.swift`, not read in full |

## Shortcuts

| Key | Action | Where |
|---|---|---|
| ⌘, | Settings | `Sources/App/OhMyAndroidApp.swift:51` |
| ⌘Q | Quit | `:53` |
| ⌘R | Refresh (Data Inspector) / re-snapshot (Layout Inspector) | `Sources/UI/Inspector/InspectorView.swift:42`, `Sources/UI/Layout/LayoutInspectorView.swift:35` |
| ⌘G | Toggle grid / guides (Layout Inspector) | `Sources/UI/Layout/LayoutInspectorView.swift:38` |
| ⇧⌘A | TalkBack order audit | `:41` |
| ⌘O | Choose design overlay | `:44` |
| ⌘C / ⇧⌘C | Copy measurement / copy as image | `:50`, `:183` |
| Arrows | nudge the measurement probe by 1 px | `:24-27` |
| Esc | clear the selection | `:28` |

## Layout Inspector (the most complex screen)

| Element | Purpose | Evidence |
|---|---|---|
| `MeasurementCanvas` | draws the frozen screenshot, view bounds, gaps/padding, and the optional design overlay | `Sources/UI/Layout/LayoutInspectorView.swift:215-320` |
| `InfoBar` | current node summary in dp, zoom, actions | `:153-214` |
| `AccessibilityListView` | ordered TalkBack stops with issue badges (no label, NAF, <48 dp target, undescribed image) | `Sources/UI/Layout/AccessibilityListView.swift:4`, `Sources/Core/Android/AccessibilityTraversal.swift:62-77` |
| dp conversion | `pixels × 160 / density`, density from `wm density` (override wins) | `Sources/Core/Android/LayoutSnapshot.swift:57-63`, `Sources/Core/Support.swift:24-27` |
| Design overlay | PNG/JPEG dropped on the window or opened with the app | `Sources/App/AppDelegate.swift:48-57`, `FileDrop` |

Windows equivalent / rethink: exact measurement tooling is portable logic (dp maths, traversal, issue
rules) but has no direct WinUI control — it becomes a custom-drawn canvas (Win2D/SkiaSharp) or an
image + adorner layer. The TalkBack audit list is a plain virtualized list. Keep the `PanelState`
model; replace glass/`NSPanel` with a normal top-most window (and drop "dock to emulator" unless there
is a Windows analogue — emulator window titles are less stable).

## Data Inspector

| Element | Purpose | Evidence |
|---|---|---|
| `PreferencesPane` | app list → XML file → typed key/value rows | `Sources/UI/Inspector/InspectorView.swift:55-84`, `Sources/Core/Android/AppData.swift:50-53` |
| `DatabasePane` | tables → 500-row pages, schema, read-only ad-hoc SQL | `Sources/UI/Inspector/InspectorView.swift:85-133`, `Sources/Core/SQLite/SQLiteSnapshot.swift:19-69` |
| Safety | read-only by construction: `run-as` reads, a local copy, `PRAGMA query_only`, no `ATTACH` | `Sources/Core/SQLite/SQLiteSnapshot.swift:63-69`, `:86` |

## Settings

| Tab | Contents | Evidence |
|---|---|---|
| General | SDK path + Choose…/Relaunch, dock toggle, open-at-login (`SMAppService`), Captures path | `Sources/App/SettingsView.swift:27-108` |
| Updates | automatic check toggle | `:109-...`, `Sources/App/UpdateController.swift:21-30` |
| AI Agents | access level (Off / Read only / Full control), last client + last used, setup instructions | `Sources/App/AgentSettingsView.swift:4-109`, `Sources/Core/AgentAccess.swift:4-31` |

## Accessibility and theming

- VoiceOver: toast messages are announced explicitly (`Sources/State/FeatureStore.swift:127`); the
  menu-bar badge image has an `accessibilityDescription` (`Sources/App/UpdateController.swift:76`).
  No `.accessibilityLabel` audit was performed → **gap**: per-cell labels are carried by the icon's
  `.help` tooltip at best (`Sources/UI/HeaderView.swift:49-50` pattern).
- Dark mode: automatic (system materials / `glassEffect`); no custom palette.
- DPI/multi-monitor: docking uses `NSScreen.screens` and `visibleFrame` per screen
  (`Sources/Windows/PanelController.swift:97-111`); no explicit scaling code.
- Keyboard-first: the panel is not fully keyboard navigable; the inspectors largely are.

## Evidence

`Sources/UI/**` (see `docs/coverage.md` for partial reads), `Sources/Windows/*.swift`,
`Sources/App/{OhMyAndroidApp,AppDelegate,SettingsView,AgentSettingsView,UpdateController}.swift`,
`Sources/State/AppModel.swift:47-70`.

## Open questions

#5 (measurement semantics), #7 (docking detection).