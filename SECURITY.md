# Security policy

## Supported versions

Only the latest release receives fixes. This is a small fork; there are no long-term branches.

## Reporting a vulnerability

**Do not open a public issue.**

Use GitHub's private advisory flow: **Security → Report a vulnerability** on
<https://github.com/bramburn/oh-my-android/security/advisories/new>, or email the maintainer listed on the
GitHub profile. Please include:

- the version (`About`/installer file name) and your Windows + WebView2 version,
- what an attacker gains, and the smallest reproduction you have,
- whether the issue is in our code or inherited from upstream (`Sources/` is not shipped here, but report it
  upstream too if it is theirs).

Expect an acknowledgement within a week. There is no bug bounty.

## What the app does with your machine

Useful context when judging a report:

| Surface | Behaviour |
|---|---|
| Processes | Exactly one: `adb.exe` from **your** Android SDK, invoked as an argv array with no shell, from `crates/adbkit` only. Commands: `version`, `devices -l`, `start-server`, `kill-server` |
| Spawned children | `adb` may start its own daemon; it is placed in a Windows Job Object so nothing outlives the app |
| Network | None. The UI is local (WebView2 + bundled assets); no telemetry, no update checks, no CDN |
| Files written | `%APPDATA%\dev.bramburn.androidpanel\config.json` (one key: your SDK folder) and the WebView2 user-data directory |
| Files read | SDK/Studio install locations, `%LOCALAPPDATA%\.android\avd` (AVD count), the config file |
| Links | Only four hard-coded `developer.android.com` URLs, opened in your system browser behind a Rust-side allowlist |

Out of scope by design: the app does not install, download or update anything, and does not modify devices.

## Scope

In scope: the Windows app (`crates/`, `src-tauri/`, `src/`), its CI workflows, and the installer.

Out of scope: the upstream macOS implementation in `Sources/` (report to
[ateymoori/oh-my-android](https://github.com/ateymoori/oh-my-android)), vulnerabilities that require an
already-compromised machine or a malicious `adb` binary the user installed themselves, and SmartScreen/AV
warnings caused by the installer being unsigned (that is a known, documented limitation, not a vulnerability:
release assets are published with SHA-256 checksums).