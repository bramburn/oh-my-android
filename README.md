# Oh My Android — Windows port fork

A fork of **[Oh My Android](https://github.com/ateymoori/oh-my-android)** (MIT, © 2026 Royan AB), a
native macOS menu-bar GUI and embedded MCP server for `adb`-driven Android testing. The goal of this
fork is to **document the product exhaustively, then re-implement it as a Windows-first GUI**, if that
decision is approved.

| | |
|---|---|
| Status | **Documentation phase. No application code has been changed.** |
| Fork point | `9f37391eef056c0e6a45327b56670f200bb62460` (upstream `main`, release `v0.4.6`) |
| Baseline tag | `upstream-baseline-9f37391` |
| Working branch | `feat/windows-gui-port` |
| Upstream / origin | `upstream` → `ateymoori/oh-my-android`, `origin` → `bramburn/oh-my-android` |
| Upstream docs | preserved verbatim in [`docs/upstream/`](docs/upstream/) |

> **Read this before assuming anything about the stack:** upstream is **Swift 6 + SwiftUI/AppKit,
> macOS 26+**, with a second executable target (`ohmyandroid-mcp`) embedded in the app bundle. It is not
> Electron, not Tauri, not .NET, and it has **no screen mirroring** and **no bundled `adb`**.
> See [`docs/repo-inventory.md`](docs/repo-inventory.md).

## What the upstream product does

A always-visible floating panel (plus an MCP server for AI agents) that flips Android test conditions in
one click and measures layout in `dp`:

- **Appearance** — dark mode, font size, display size, language/locale incl. RTL and pseudo-locales.
- **Accessibility** — TalkBack, bold text, invert colors, and a TalkBack-order audit with missing-label
  and 48 dp touch-target checks.
- **Layout** — a dp measurement inspector with design-overlay comparison, layout bounds, overdraw, GPU bars.
- **App** — restart, clear data, force stop, uninstall, reset permissions, deep links, APK install.
- **Data** — read-only SharedPreferences and SQLite of debuggable builds.
- **Simulate** — network profiles, GPS, battery, incoming call/SMS (emulator console).
- **Agents** — an MCP server with 18 tools and 3 prompts over stdio.

## Documentation index

| Document | What it answers |
|---|---|
| [`docs/repo-inventory.md`](docs/repo-inventory.md) | Stack, file table, must-read order, Windows-port disposition per directory |
| [`docs/architecture/00-overview.md`](docs/architecture/00-overview.md) | Purpose, capabilities, process model, "how it actually works" |
| [`docs/architecture/01-c4-context.md`](docs/architecture/01-c4-context.md) | Actors, systems, trust boundaries |
| [`docs/architecture/02-containers.md`](docs/architecture/02-containers.md) | The two deployables, shared code, windows |
| [`docs/architecture/03-components.md`](docs/architecture/03-components.md) | Every component: responsibility, surface, state, failure modes |
| [`docs/architecture/04-data-and-state.md`](docs/architecture/04-data-and-state.md) | Models, every persisted key, on-disk artefacts |
| [`docs/architecture/05-concurrency-and-lifecycle.md`](docs/architecture/05-concurrency-and-lifecycle.md) | Isolation, timeouts, retries, hotplug, startup/shutdown |
| [`docs/architecture/06-adb-integration.md`](docs/architecture/06-adb-integration.md) | **Every adb call**, quoting hazards, what is not implemented |
| [`docs/architecture/07-mirroring-and-control.md`](docs/architecture/07-mirroring-and-control.md) | Why there is no mirroring, and what that means for the port |
| [`docs/architecture/08-ui-ux-map.md`](docs/architecture/08-ui-ux-map.md) | Every screen, state, shortcut, and a Windows rethink note |
| [`docs/architecture/09-cross-cutting.md`](docs/architecture/09-cross-cutting.md) | Logging, errors, network/telemetry audit, security, packaging |
| [`docs/architecture/10-dependencies-and-licenses.md`](docs/architecture/10-dependencies-and-licenses.md) | Sparkle, platform-tools terms, licence obligations |
| [`docs/architecture/11-build-run-test.md`](docs/architecture/11-build-run-test.md) | Toolchain, commands, test inventory and gaps (all `UNVERIFIED`) |
| [`docs/architecture/12-risks-and-tech-debt.md`](docs/architecture/12-risks-and-tech-debt.md) | Ranked risks to the port + debt worth fixing |
| [`docs/feature-matrix.md`](docs/feature-matrix.md) | Every feature with complexity, Windows feasibility and MVP tier |
| [`docs/coverage.md`](docs/coverage.md) | What was read, skimmed, skipped |
| [`docs/assumptions.md`](docs/assumptions.md) | Assumptions with confidence and blast radius |
| [`docs/open-questions.md`](docs/open-questions.md) | Open questions with the exact file/command to resolve each |
| [`docs/git-setup.md`](docs/git-setup.md) | Remotes, branches, tags, sync-upstream playbook |
| [`docs/legal.md`](docs/legal.md) | Licence, attribution, bundled-binary rights, naming |
| [`docs/CONTRIBUTING-fork.md`](docs/CONTRIBUTING-fork.md) | Fork conventions: branches, commits, doc-drift rule, PR checklist |

## How to build the original (unchanged)

Requirements: macOS 26, Xcode 26, and an Android SDK that provides `platform-tools/adb`.

```sh
brew install xcodegen
xcodegen generate          # the .xcodeproj is generated and git-ignored
open OhMyAndroid.xcodeproj # run the OhMyAndroid scheme
```

Release build (no signing identity needed locally):

```sh
xcodebuild -project OhMyAndroid.xcodeproj -scheme OhMyAndroid -configuration Release \
  -derivedDataPath build/DerivedData CODE_SIGN_IDENTITY=- build
Scripts/test-mcp.py "build/DerivedData/Build/Products/Release/Oh My Android.app/Contents/MacOS/ohmyandroid-mcp"
```

`UNVERIFIED` on this fork's machine (no macOS, no Xcode, no `adb` available). Sources:
`CONTRIBUTING.md:9-12`, `.github/workflows/build.yml:23-31`.

## Safety

Nothing in this fork's documentation phase touched a device. The only `adb` behaviour documented here was
read from source. See `docs/CONTRIBUTING-fork.md` for the ADB safety rules that apply to future work.

## Licence

MIT, as upstream. See [`LICENSE`](LICENSE) (unchanged) and [`NOTICE.md`](NOTICE.md).