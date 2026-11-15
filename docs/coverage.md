# Coverage — what was actually read

Honesty file. Baseline `9f37391`.

## Read in full (with line numbers)

| File | Lines |
|---|---|
| `project.yml` | all |
| `Sources/Core/Android/ADBClient.swift` | all |
| `Sources/Core/Shell/ShellRunner.swift` | all |
| `Sources/Core/Android/AndroidSDK.swift` | all |
| `Sources/Core/Android/DeviceTracker.swift` | all |
| `Sources/Core/Android/Device.swift` | all |
| `Sources/Core/Android/DeviceInfo.swift` | all |
| `Sources/Core/Android/EmulatorLauncher.swift` | all |
| `Sources/Core/Android/AppData.swift` | all |
| `Sources/Core/Android/LayoutSnapshot.swift` | all |
| `Sources/Core/Android/AccessibilityTraversal.swift` | all |
| `Sources/Core/Android/ForegroundAppReader.swift` | all |
| `Sources/Core/SQLite/SQLiteSnapshot.swift` | all |
| `Sources/Core/Support.swift` | all |
| `Sources/Core/AgentAccess.swift` | all |
| `Sources/Features/Feature.swift` | all |
| `Sources/Features/FeatureCatalog.swift` | all |
| `Sources/Features/DisplayFeatures.swift` | all |
| `Sources/Features/AccessibilityFeatures.swift` | all |
| `Sources/Features/DebugFeatures.swift` | all |
| `Sources/Features/InputFeatures.swift` | all |
| `Sources/Features/GenericFeatures.swift` | all |
| `Sources/Features/AppFeatures.swift` | all |
| `Sources/Features/CaptureFeatures.swift` | all |
| `Sources/Features/BatchReader.swift` | declarations + logic |
| `Sources/State/AppModel.swift` | all |
| `Sources/State/DeviceStore.swift` | all |
| `Sources/State/FeatureStore.swift` | all |
| `Sources/State/EmulatorStore.swift` | all |
| `Sources/App/AppDelegate.swift` | all |
| `Sources/App/UpdateController.swift` | all |
| `Sources/App/OhMyAndroidApp.swift` | all |
| `Sources/App/SettingsView.swift` | 1-60 |
| `Sources/Windows/PanelController.swift` | all |
| `Sources/Windows/FloatingPanel.swift` | 1-50 |
| `Sources/UI/PanelRootView.swift` | 1-60 |
| `Sources/UI/Theme.swift` | all |
| `Sources/MCP/main.swift` | all |
| `Sources/MCP/Server/MCPServer.swift` | all |
| `Sources/MCP/Server/Catalog.swift` | all |
| `Sources/MCP/Tools/Tool.swift` | all |
| `Sources/MCP/Tools/ToolEnvironment.swift` | all |
| `Sources/MCP/Tools/AppTools.swift` | 110-151 (logcat) |
| `Sources/MCP/Tools/*.swift` | tool names/descriptions via `rg`, not full bodies |
| `Scripts/release.sh` | 1-120 |
| `Scripts/test-mcp.py` | 1-40, `wc -l` |
| `mcpb/manifest.json`, `.claude-plugin/plugin.json` | all |
| `LICENSE`, `SECURITY.md`, `CONTRIBUTING.md`, `README.md` | head |

## Read partially

| File | What was read | What was not |
|---|---|---|
| `Sources/UI/Layout/LayoutInspectorView.swift` (319 L) | structure, key handlers, drawing helper names | full drawing math, overlay alignment logic |
| `Sources/UI/FeatureCell.swift` (181 L) | declaration map | per-control rendering details |
| `Sources/UI/Inspector/InspectorView.swift` (133 L) | declaration map | pane internals |
| `Sources/State/{Inspector,LayoutInspector,DeviceInfo}Store.swift` | declaration map | body |
| `Sources/MCP/Tools/{Device,Screen,Input,App,Data,Emulator}Tools.swift` | names, descriptions, `rg` line-level for every adb call | full parameter/impl bodies |
| `Sources/Windows/{ToolWindow,InspectorWindowController,LayoutInspectorWindowController,EmulatorWindowTracker}.swift` | declarations | body |
| `Sources/Core/Imaging/*` | existence only | algorithms |
| `Sources/MCP/Format/UITreeText.swift`, `Sources/MCP/Protocol/JSONValue.swift` | declaration map | body |
| `README.md` | 1-60 | rest (marketing, install, FAQ) |
| `skills/android-ui-check/SKILL.md`, `server.json`, `glama.json` | existence | content |

## Deliberately skipped

- `docs/*.jpg`, `docs/*.gif`, `docs/social-preview.png` — binary marketing assets (2.9 MB total).
- `Sources/App/Assets.xcassets/**` — icon PNGs.
- `.github/ISSUE_TEMPLATE/**`, `pull_request_template.md`.
- Git internals beyond `git log`/`branch`/`remote`/`tag`.

## Gaps this leaves

1. Exact SwiftUI layout of `FeatureCell` popovers (stepper/choice/text action) — behaviour inferred
   from the protocols, not the pixels. → docs/architecture/08-ui-ux-map.md marks it.
2. Per-tool argument validation for the 18 MCP tools — read via `rg` of `name:` and adb call sites,
   not line-by-line. → docs/open-questions.md #4.
3. `LayoutInspectorStore` measurement/overlay maths — not verified.
   → docs/open-questions.md #5.
4. Ordering/implementation of `Imaging` downscaling (claim "1 px = 1 dp" is taken from the README
   and tool descriptions, not from the code). → docs/assumptions.md A4.
5. Nothing was executed. No build, no test, no `adb` call. Every command reference in these docs is
   `UNVERIFIED` unless it is a literal line of a committed script.