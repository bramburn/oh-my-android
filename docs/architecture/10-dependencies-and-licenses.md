# 10 — Dependencies and licences

## Third-party dependencies

| Dependency | Version | Purpose | Licence | Bundled? | Windows availability | Weight | Replacement candidate | Risk |
|---|---|---|---|---|---|---|---|---|
| **Sparkle** | 2.10.0 (pinned `exactVersion`) | In-app updates (EdDSA-signed appcast) | MIT | Yes, `Sparkle.framework` inside the app | **macOS only** | ~3–5 MB | Velopack (MIT), NetSparkle (MIT), WinSparkle (MIT), or "manual download" | Low for upstream; a Windows port must choose a different updater |

Everything else is a system framework: SwiftUI, AppKit, Foundation, `Synchronization` (`Mutex`),
`SQLite3`, `ServiceManagement`, `UniformTypeIdentifiers`, `Observation`
(`import` lines across `Sources`).

## Build/dev tooling (not shipped)

| Tool | Role | Licence | Evidence |
|---|---|---|---|
| XcodeGen | generates `OhMyAndroid.xcodeproj` from `project.yml` | MIT | `.github/workflows/build.yml:23-25`, `CONTRIBUTING.md:9-10` |
| Xcode 26 / `xcodebuild` | compiler + packaging | Apple EULA | `CONTRIBUTING.md:6-7` |
| `Scripts/test-mcp.py` | MCP protocol test (Python 3, stdlib only) | in-repo (MIT) | `Scripts/test-mcp.py:1-8` |
| `Scripts/make-icon.swift` | generates the icon | in-repo | file presence |

## External binaries that this app *does not* ship but *requires*

| Binary | Where it comes from | Licence/terms | Evidence |
|---|---|---|---|
| `adb` (platform-tools) | the user's Android SDK | Google Android SDK terms (platform-tools binaries are **not** freely redistributable without the SDK terms) | `Sources/Core/Android/AndroidSDK.swift:3-6` ("Nothing is bundled; Google's license forbids redistribution") |
| `emulator` | the user's Android SDK | same | `Sources/Core/Android/EmulatorLauncher.swift:3-6` |
| `uiautomator`, `settings`, `cmd`, `pm`, `am`, `input`, `run-as`, `logcat` | the device image | Android Open Source Project (Apache-2.0), already on the device | used via `adb shell` |

`UNVERIFIED`: the exact wording of Google's current platform-tools redistribution terms. Needed before
any Windows port that bundles `platform-tools` (`docs/open-questions.md` #3). Reference:
`platform-tools/NOTICE.txt` in any SDK install.

## Repo licence

| Aspect | Value | Evidence |
|---|---|---|
| Licence | MIT | `LICENSE:1` |
| Copyright | © 2026 Royan AB | `LICENSE:3` |
| Fork obligations | keep the copyright notice + permission notice in all copies/substantial portions | `LICENSE:4-20` (MIT terms) |
| Copyleft | none | — |
| Relicensing | cannot remove the MIT notice from upstream code; a fork may add its own notice around it | — |
| Trademarks / names | "Oh My Android", "Royan AB", bundle id `se.royan.ohmyandroid`, the app icon, the Homebrew tap name and the GitHub release feed are the author's identity. A fork should use its own name and, if it keeps in-app updates, its own feed + signing key. | `project.yml:57-58`, `Scripts/release.sh:88-111`, `mcpb/manifest.json` |
| Redistribution of vendor names in metadata | `mcpb/manifest.json` and `.claude-plugin/plugin.json` point at the upstream author and repo; a fork must change `repository`, `homepage`, `author`, `support` | `mcpb/manifest.json:8-17`, `.claude-plugin/plugin.json:3-8` |

## Licence risk summary for a Windows port

1. **No copyleft anywhere.** MIT in, MIT out. Clean-room rewrite is *not* legally required; a
   reimplementation in another language of MIT-licensed behaviour only needs the MIT notice preserved
   in the fork (which the fork's `NOTICE.md` does).
2. **The only real licensing hazard is bundling Google's platform-tools.** That is a distribution
   decision, not a code decision, and it applies to *any* Windows ADB GUI.
3. **Naming.** Do not ship as "Oh My Android" pointing at the upstream feed/keys, or users will get
   updates signed by someone else — or silently stop getting them.

## Evidence

`project.yml:26-31`, `LICENSE`, `Scripts/release.sh`, `mcpb/manifest.json`,
`.claude-plugin/plugin.json`, `Sources/Core/Android/AndroidSDK.swift`, `CONTRIBUTING.md`.

## Open questions

#3 (platform-tools terms).