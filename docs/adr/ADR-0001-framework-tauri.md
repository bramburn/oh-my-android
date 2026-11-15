# ADR-0001 — Windows GUI framework: Tauri v2

- Status: **Accepted** (user decision, Gate 2)
- Date: 2026-11-15
- Deciders: bramburn (product owner)
- Supersedes: none

## Context

Upstream is **Swift 6 + SwiftUI/AppKit, macOS 26+**, 6 605 lines across 98 text files, one third-party
dependency (Sparkle), `project.yml:10-11`. There is **no portable UI layer**: the views are SwiftUI
(`Sources/UI/**`), the panels are AppKit `NSPanel` subclasses (`Sources/Windows/**`), the lifecycle is
SwiftUI `App`/`@Observable` (`Sources/App/OhMyAndroidApp.swift`, `Sources/State/AppModel.swift`).
A Windows GUI is therefore a **rewrite of the entire presentation layer**, not a port.

What *is* portable is the **domain behaviour**: which adb command runs, in what order, with which
quoting, and how the output is parsed (`docs/architecture/06-adb-integration.md`) — plus the UI state
model, which is unusually good (`docs/architecture/08-ui-ux-map.md`).

The product needs, in order of architectural weight:

1. reliable child-process control (`adb.exe` lifecycle, kill-on-exit, no console flashes);
2. low-latency, event-driven device state (raw `host:track-devices` socket);
3. a **very cheap idle** menu-bar/tray panel that appears instantly;
4. an installer that works without admin rights;
5. *(contested)* H.264 video into the UI at ≤120 ms — `ADR-0002`.

## Decision

**We will build the Windows app with Tauri v2 (Rust backend, WebView2, TypeScript frontend), pinned as in
`docs/windows-port/10-version-pin.md`, with Svelte 5 + Vite + Tailwind for the UI.**

## Consequences

### Positive

- Process control is native Rust: `std::process`/`tokio::process` + Win32 `CREATE_NO_WINDOW` + Job Objects.
  No Node child-process layer, no Electron runtime.
- Idle cost is a single WebView2 instance; the panel can be a small always-on-top window
  (`WS_EX_NOACTIVATE`/`WS_EX_TOPMOST` via the `windows` crate), matching upstream's
  `Sources/Windows/FloatingPanel.swift:22-40`.
- WebView2 ships with Windows 10 22H2/11 in practice, and Tauri can embed a bootstrapper or a fixed
  runtime (`WebviewInstallMode` has `downloadBootstrapper | embedBootstrapper | offlineInstaller |
  fixedRuntime`, verified in the config schema: https://schema.tauri.app/config/2).
- Installer size fits the ≤25 MB budget: Tauri NSIS installers are typically 3–8 MB `ESTIMATE`, leaving
  room for `platform-tools`.
- One IPC boundary that can be typed end-to-end, and a Rust crate (`crates/adbkit`) that is testable
  without any UI.
- Video, if M3 happens, can be decoded by the WebView2 engine via **WebCodecs** — no ffmpeg bundle
  (`SPIKE S3`).

### Negative / accepted costs

- **Two languages and two build systems.** Mitigated by keeping all device logic in Rust and all UI logic
  in TS, with a single documented contract (`12-ipc-contract.md`).
- **WebView2 dependency** on old/offline machines; enterprise offline installs need
  `webviewInstallMode: offlineInstaller` or `fixedRuntime` (`16-build-package-release.md`).
- **WebCodecs support inside WebView2 is unverified** — the deciding risk for mirroring (`SPIKE S3`).
- WebView2's JIT/GC makes a 60 fps UI in a webview possible but not free; a native UI would be leaner.
- Debugging spans two debuggers (Rust + DevTools).

### Neutral / follow-ups

- A Cargo workspace with `crates/adbkit`, `crates/mirror`, `src-tauri` (`05-repo-restructure.md`).
- A CI grep guard forbidding `Command::new` outside `crates/adbkit` (`15-plugin-and-security.md` §6).

## Alternatives considered

| Option | Why not chosen |
|---|---|
| **.NET 8 + WPF (or WinUI 3)** | Best raw Windows integration (native windows, Media Foundation for H.264, no WebView2), single language. Not chosen by the user; recorded as the strongest alternative if WebCodecs fails and mirroring is required. |
| **.NET + Avalonia** | Cross-platform without being Windows-native; XAML learning cost without native HWND/Job control advantage over WPF. |
| **Electron** | 100 MB+ runtime, Node child-process layer, worse idle cost; contradicts the ≤25 MB installer budget. |
| **Flutter desktop** | Good UI, weak Win32 interop for window docking/`WS_EX_NOACTIVATE`, no benefit over Tauri here. |
| **Native C++/Win32** | Best possible panel/video, worst velocity and testability for a solo maintainer. |
| **Keep upstream Swift + a Windows VM** | Not a product. |

## Evidence

- Upstream stack: `project.yml:10-11`, `Sources/App/OhMyAndroidApp.swift`, `Sources/Windows/FloatingPanel.swift:22-40`.
- Tauri config schema (resources, externalBin, WebviewInstallMode):
  https://schema.tauri.app/config/2 — retrieved 2026-11-15.
- `tauri` 2.12.5 is the current stable release on crates.io — https://crates.io/api/v1/crates/tauri.
- Spike results: `20-spikes.md` (S3 decides the video path, S4 the process path).