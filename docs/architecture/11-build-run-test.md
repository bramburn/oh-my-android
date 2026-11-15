# 11 — Build, run, test

**Nothing in this document was executed on this machine.** There is no macOS, no Xcode and no `adb`
here; the repo was cloned and read on Windows. Every command is copied from a committed file and marked
`UNVERIFIED`.

## Toolchain

| Requirement | Version | Evidence | Status |
|---|---|---|---|
| macOS | **26.0+** (Tahoe) | `project.yml:4-5`, `:11`, `README.md:14` | `UNVERIFIED` |
| Xcode | 26 | `CONTRIBUTING.md:6-7` | `UNVERIFIED` |
| Swift | 6.0 language mode, strict concurrency, warnings = errors | `project.yml:10`, `:20` | `UNVERIFIED` |
| XcodeGen | latest via Homebrew (not pinned) | `CONTRIBUTING.md:9`, `.github/workflows/build.yml:23` | `UNVERIFIED` |
| Android SDK | Android Studio **or** `brew install --cask android-platform-tools`; only `platform-tools/adb` (+ optionally `emulator`) is used | `CONTRIBUTING.md:6-7`, `Sources/Core/Android/AndroidSDK.swift:13-32` | `UNVERIFIED` |
| Python | 3 (stdlib only) for `Scripts/test-mcp.py` | `Scripts/test-mcp.py:1-8`, `.github/workflows/build.yml:31` | `UNVERIFIED` |
| Apple Developer account | only for releases (notarytool profile + Developer ID + Sparkle EdDSA key) | `Scripts/release.sh:5-9`, `:33-44`, `:65-70` | `UNVERIFIED` |

## Environment variables

| Variable | Used by | Default | Evidence |
|---|---|---|---|
| `ANDROID_HOME` | SDK discovery | unset | `Sources/Core/Android/AndroidSDK.swift:15-18` |
| `ANDROID_SDK_ROOT` | SDK discovery (after `ANDROID_HOME`) | unset | same |
| `ANDROID_ADB_SERVER_PORT` | the `host:track-devices` socket only | `5037` | `Sources/Core/Android/DeviceTracker.swift:29-31` |
| `TEAM_ID` | release signing (required) | none | `Scripts/release.sh:16` |
| `NOTARY_PROFILE` | notarytool keychain profile | `ohmyandroid` | `Scripts/release.sh:17` |
| `REPO` | appcast URL prefix | `ateymoori/oh-my-android` | `Scripts/release.sh:18` |
| `NOTES` | Markdown release notes file | unset | `Scripts/release.sh:9-12` |

## Dev loop

```sh
brew install xcodegen
xcodegen generate            # writes OhMyAndroid.xcodeproj (git-ignored)
open OhMyAndroid.xcodeproj   # run the OhMyAndroid scheme
```
Source: `CONTRIBUTING.md:9-12`. Local builds are ad-hoc signed (`project.yml:15`), so no developer
account is needed to run. `UNVERIFIED`.

Debugging notes from the code that matter on a fresh machine:

- If the panel says "Android SDK Not Found", the app looked in 8 places and none had an executable
  `platform-tools/adb` (`Sources/Core/Android/AndroidSDK.swift:13-32`). Set the folder in Settings;
  it persists as `sdk.customRoot` (`:11`).
- The MCP server locates the SDK with the **same** chain, plus the app's saved choice, because it reads
  the app's preferences domain (`Sources/MCP/main.swift:30`).
- There is no debug logging; attach Xcode to the running app (see `docs/architecture/09-cross-cutting.md`).

## Build

```sh
xcodegen generate
xcodebuild -project OhMyAndroid.xcodeproj -scheme OhMyAndroid -configuration Release \
  -derivedDataPath build/DerivedData CODE_SIGN_IDENTITY=- build
```
Source: `.github/workflows/build.yml:23-30`. Two targets are built: `OhMyAndroid` (app) and
`ohmyandroid-mcp` (embedded executable) (`project.yml:32-93`). `UNVERIFIED`.

## Test inventory and coverage gaps

| Test | What it covers | Runs where | Evidence |
|---|---|---|---|
| `Scripts/test-mcp.py <mcp-binary>` | JSON-RPC conformance: legacy `initialize`, modern `_meta` era, `tools/list`, `prompts/list`, tool schemas, error codes, "stdout carries nothing but JSON-RPC" | CI (`macos-26`) + release gate | `Scripts/test-mcp.py:1-8`, `.github/workflows/build.yml:30-31`, `Scripts/release.sh:59` |
| `xcodebuild … build` with warnings-as-errors | compile-level correctness + strict concurrency | CI | `project.yml:20`, `.github/workflows/build.yml:28-30` |

**Gaps (all of them):** no unit tests, no UI tests, no fixture-based parser tests, no fake-adb harness,
no integration test against a device or emulator, no coverage measurement. `rg -l "XCTest|import Testing" Sources`
→ 0 hits. So the parsers with the most risk — `adb devices -l` lines, `wm size`/`wm density`,
`uiautomator` XML, `logfmt` of `logcat`, SharedPreferences XML — are verified only by hand.

## CI

| Workflow | Trigger | Jobs | Evidence |
|---|---|---|---|
| `build.yml` | push to `main`, PR | `macos-26`: xcodegen → Release build → MCP protocol test | `.github/workflows/build.yml:1-34` |
| `downloads.yml` | cron 6-hourly, release published, manual | `ubuntu-latest`: sums release asset download counts into `downloads.json` on the `badges` branch | `.github/workflows/downloads.yml:1-45` |

Both pin `actions/checkout` by SHA (supply-chain hygiene). No caching, no matrix, no Windows runner.

## Release

```sh
TEAM_ID=<team> [NOTES=notes.md] Scripts/release.sh [--skip-notarize]
```
`Scripts/release.sh:1-11`. Steps, all `UNVERIFIED`:
1. `xcodegen generate`, clean build with `Developer ID Application` (`:23-31`).
2. Sign Sparkle's nested helpers inside-out, then re-seal the app (`:33-44`).
3. Refuse to ship if `get-task-allow` is set or a timestamp is missing (`:49-58`).
4. Verify signature, run the MCP test (`:58-59`).
5. Zip, notarize, staple, re-zip, `spctl` check (`:61-70`).
6. Generate + verify the EdDSA appcast for the `-update.zip` (`:72-85`).
7. Write the Homebrew cask with the sha256 (`:87-111`).
8. Build the `.mcpb` (server only) and bump its manifest version (`:112-120`).

## Lint / format

None configured. No SwiftLint/SwiftFormat config, no `swift-format` file. Style is enforced only by
`SWIFT_TREAT_WARNINGS_AS_ERRORS: YES` (`project.yml:20`) and the rules in `CONTRIBUTING.md:18-32`.

## Evidence

`project.yml`, `CONTRIBUTING.md`, `.github/workflows/*.yml`, `Scripts/release.sh`, `Scripts/test-mcp.py`.
None of it was executed in this session.