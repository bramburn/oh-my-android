# 16 — Build, package, release

## 1. Commands (as built)

Verified on Windows 11 / WebView2 157. There is no `npm run lint` or `npm run test` yet — `svelte-check`
is the frontend gate and is wired into `npm run build`.

| Task | Command | Output |
|---|---|---|
| Install frontend deps | `npm ci` | exact lockfile install |
| Dev (Vite HMR + Tauri window) | `npm run tauri dev` | window on `http://localhost:1420` |
| Type check | `npm run check` | `svelte-check`, 0 errors/warnings |
| Rust gates | `cargo fmt --all --check` · `cargo clippy --workspace --all-targets -- -D warnings` | |
| Tests | `cargo test --workspace` | 46 tests, no device needed |
| Frontend build | `npm run build` | `svelte-check` + `dist/` |
| Bundle installer | `npm run tauri build` | `target/release/bundle/nsis/*-setup.exe` (~1.6 MiB) |
| Silent install / uninstall | `& ".\*_x64-setup.exe" /S` · `uninstall.exe /S` | per-user, no elevation |

MSI and updater artifacts are **not** configured: `bundle.targets = ["nsis"]`, and there is no
`createUpdaterArtifacts` (no update feed yet — ADR-0007).

## 2. `tauri.conf.json` skeleton (annotated)

Every key below exists in the v2 schema — https://schema.tauri.app/config/2 (retrieved 2026-11-15).

```jsonc
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "ADB Panel",                    // identity: NOT "Oh My Android" (docs/legal.md §1)
  "version": "0.1.0",
  "identifier": "io.github.bramburn.adbpanel",    // never se.royan.ohmyandroid
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "app": {
    "withGlobalTauri": false,                     // smaller attack surface
    "windows": [
      { "label": "panel", "title": "ADB Panel", "width": 320, "height": 720,
        "minWidth": 260, "resizable": true, "alwaysOnTop": true,
        "decorations": false, "transparent": false, "skipTaskbar": true,
        "visible": false,                         // shown by Rust once state is loaded
        "devtools": false },
      { "label": "settings", "title": "Settings", "width": 640, "height": 560,
        "resizable": true, "visible": false },
      { "label": "logcat", "title": "Logcat", "width": 900, "height": 600, "visible": false }
    ],
    "security": {
      "csp": "default-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob: asset: http://asset.localhost; media-src blob:; connect-src 'self' ipc: http://ipc.localhost http://127.0.0.1:*; frame-src 'none'; object-src 'none'",
      "capabilities": ["panel", "settings", "logcat", "files"]   // explicit → nothing auto-enabled
    },
    "trayIcon": { "id": "main", "iconPath": "icons/tray.png", "iconAsTemplate": false }
  },
  "bundle": {
    "active": true,
    "targets": ["nsis"],                          // MSI only on demand
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.ico"],
    "resources": { "resources/platform-tools/": "platform-tools/" },  // whole folder: adb.exe + AdbWin*.dll
    "licenseFile": "../LICENSE",
    "category": "DeveloperTool",
    "shortDescription": "Android test panel for Windows",
    "windows": {
      "webviewInstallMode": { "type": "embedBootstrapper", "silent": true },
      "allowDowngrades": false,
      "nsis": { "installMode": "currentUser", "compression": "lzma", "languages": ["English"] }
    }
  },
  "plugins": {
    "updater": { "endpoints": ["https://github.com/<fork>/releases/latest/download/latest.json"],
                 "pubkey": "<new minisign public key>" }
  }
}
```

### Field notes

| Field | Why this value |
|---|---|
| `bundle.resources` as a **map** | `{"src/": "platform-tools/"}` keeps the folder name stable in `$RESOURCES`; a plain list would preserve the relative path under `resources/` — both are legal (`VERIFIED`, `BundleResources`) |
| **not** `externalBin` | `externalBin` requires the `binary-name-x86_64-pc-windows-msvc.exe` suffix and is for *executables you invoke directly*; adb needs its sibling DLLs, so a resource folder is correct (`VERIFIED`, `BundleConfig.externalBin` docs in the schema) |
| `webviewInstallMode: embedBootstrapper` | works offline-ish and keeps the installer small; `offlineInstaller` (+~130 MB `ESTIMATE`) only if enterprise customers demand it; `fixedRuntime` pinned only if a WebCodecs bug requires a specific Edge version |
| `nsis.installMode: currentUser` | no admin prompt; matches the "panel tool" nature. `perMachine` for enterprise |
| `decorations: false` on the panel | the panel is a floating tool; drag region in Svelte. Settings/Logcat keep native chrome |
| `skipTaskbar: true` on the panel | tray + panel only, like upstream's menu-bar app |
| `visible: false` | prevents a white flash before the first `devices://changed` |

## 3. Bundling decisions

| Question | Decision | Consequence |
|---|---|---|
| Bundle `platform-tools`? | **No, v1** — reuse upstream's "bring your own SDK" model | avoids Google SDK-terms redistribution risk (`docs/legal.md` §2). First-run onboarding asks for the SDK folder or downloads nothing |
| If bundled later | copy from the user's SDK at **build time** on a machine that has accepted the terms, and record it in `NOTICE.md` | installer +~10 MB |
| Bundle `scrcpy`? | only if ADR-0002 picks A/B/C | +~6 MB, Apache-2.0 NOTICE entry |
| Bundle `scrcpy-server.jar` (A) | yes, as a resource | must match `crates/mirror`'s pinned protocol constant |
| VC++ runtime | `bundleVCRuntime: true` if the MSVC toolchain links dynamically | otherwise the installer fails on clean machines |
| Portable zip | not produced (D6: installer, not portable) | |

## 4. Size budget

| Part | `ESTIMATE` |
|---|---|
| Rust binary (release, LTO, strip) | 6–9 MB |
| Frontend assets | ≤2 MB |
| WebView2 bootstrapper (embedded) | ~1.5 MB |
| `platform-tools` (if bundled) | ~10 MB |
| scrcpy (if bundled) | ~6 MB |
| **Installer, v1 (no platform-tools)** | **~10–13 MB** ✅ ≤25 MB |
| **Installer, with platform-tools + scrcpy** | **~26–28 MB** ⚠ over budget → ship the "bring your own" variant |

## 5. Installer: NSIS vs MSI

| | NSIS | MSI (WiX) |
|---|---|---|
| Per-user, no admin | ✅ default `currentUser` | needs `enableElevatedUpdateTask`/per-machine |
| Size | smaller | larger |
| Enterprise deployment (GPO/Intune) | harder | ✅ expected |
| Auto-updater support | ✅ first-class | ✅ but elevated updates need a task |
| **Choice** | **primary** | build on demand for enterprise; CI produces both once M6 starts |

## 6. Signing

| Option | Cost `ESTIMATE` | SmartScreen |
|---|---|---|
| Unsigned | 0 | full warning wall; unacceptable for a public release |
| **Azure Trusted Signing** | ~$10/month, needs identity validation | good, cheap for a solo dev |
| OV certificate (HSM/token) | $200–400/yr | reputation builds over time |
| EV certificate | $300–600/yr + hardware | immediate reputation |

Recommendation: **Azure Trusted Signing**, with `bundle.windows.signCommand` wired in CI. Until then, every
build is clearly marked "unsigned dev build" in the About dialog. Verify current pricing/eligibility at M6 —
`UNVERIFIED`.

## 7. CI and CD (as built)

Three workflows live in `.github/workflows/`:

| Workflow | Trigger | What it does |
|---|---|---|
| `windows.yml` | `push` to `main`, **every** `pull_request`, `workflow_dispatch` | frontend build (includes `svelte-check`) → fmt → clippy → tests → 3 guards → NSIS build → uploads the installer as an artifact (14-day retention) and writes its size to the step summary |
| `release.yml` | tag `v*` | asserts tag == `tauri.conf.json` == `package.json`, runs the full gate set, builds the installer, writes `SHA256SUMS.txt`, publishes a GitHub release with both files |
| `build.yml` | upstream's macOS workflow | untouched, and still green in this fork (2 min). **Not required** for merges |

Deliberate choices:

- **No `paths:` filter on `pull_request`.** `windows` is a required status check; a filtered workflow that
  does not run would block every merge.
- **Job name is the check context**: `windows`. Do not rename the job without updating the ruleset.
- Action SHAs are pinned; `permissions: contents: read` everywhere except the release job (`write`).
- The frontend is built **before** any cargo step: `tauri::generate_context!` embeds `dist/`.
- `downloads.yml` (upstream's macOS release-asset counter) was **removed**: it wrote to a `badges` branch
  that this fork does not have, and it counted asset names this fork never publishes.

Guards, all failing the build:

| Guard | Rule |
|---|---|
| `Command::new` outside `crates/adbkit` | invariant #1 in `AGENTS.md` |
| Tauri v1 APIs (`tauri::api::`, `allowlist`) | v2 only |
| Remote asset URLs in `src/` + `index.html` | offline-first (ADR-0009), only four docs URLs allowed |

## 8. Release procedure

1. Bump `version` in **both** `src-tauri/tauri.conf.json` and `package.json`; commit to `main` via PR.
2. Re-verify `10-version-pin.md`; lockfiles committed; `NOTICE.md` current; `LICENSE` untouched.
3. Tag and push — this is the release trigger:

   ```powershell
   git switch main && git pull --ff-only
   git tag v0.1.1
   git push origin v0.1.1
   ```

4. `release.yml` refuses to build if the tag and either manifest disagree, then runs fmt/clippy/tests,
   builds the installer **once**, and publishes the GitHub release with `*-setup.exe` + `SHA256SUMS.txt`
   and the static body from `.github/release-body.md` (install instructions, requirements, known gaps).
5. Smoke-test the published asset on a machine that has never had the app: install → run → uninstall.

Checklist before tagging:

- [ ] Installer tested on clean Win10 22H2 (no WebView2) and Win11.
- [ ] Install → run → uninstall leaves no orphan `adb.exe` and no leftover service.
- [ ] SmartScreen status recorded (unsigned builds warn; checksums published instead).
- [ ] Size and cold start measured and compared to `00-target-and-constraints.md` §3.
- [ ] Docs updated (doc-drift rule): README, this file, `21-mvp-log.md` if behaviour moved.

## 9. Repository governance

`main` is protected by a **ruleset** (not classic protection: personal repositories cannot restrict
pushing to individual users there). Ruleset `main`:

| Rule | Value |
|---|---|
| Required check | `windows`, strict (branch must be up to date) |
| Pull request | 1 approval, stale reviews dismissed, threads must resolve |
| History | linear; force-push and deletion blocked |
| Bypass | **only** `bramburn` (`actor_type: User`), mode `always` |

So: the maintainer can push directly; everyone else forks, opens a PR, and needs the `windows` check plus
one approval. Merge strategy is **squash only**, and branches are deleted on merge (repository settings).
Exact commands and how to read the ruleset back: `docs/git-setup.md` §3.
