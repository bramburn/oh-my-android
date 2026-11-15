# 18 — Plan and backlog

Estimates are `ESTIMATE` (one competent dev, familiar with Rust and Svelte, no device automation
infrastructure yet). Hours exclude review/rework. Milestones are **spike-first and dependency-ordered**:
M3 does not start until S1–S3 report.

## M0 — Skeleton, adbkit, device list, CI (≈34 h)

| ID | Task | Deps | h | Files | Acceptance | Test |
|---|---|---|---|---|---|---|
| M0-1 | Workspace scaffold: `Cargo.toml`, `rust-toolchain.toml`, `.nvmrc`, lint configs | — | 3 | root | `cargo build --workspace` + `npm ci` clean on Windows | CI |
| M0-2 | Tauri shell: `src-tauri` with panel/settings/logcat windows, capabilities, CSP | M0-1 | 4 | `src-tauri/**` | app launches, panel shows a hardcoded state | manual |
| M0-3 | `crates/adbkit`: `AdbRequest`/`AdbTransport` traits, `ProcessAdbTransport`, `shell_quote` | M0-1 | 5 | `crates/adbkit/src/**` | queued/echoed args match golden | unit |
| M0-4 | Windows process layer: `CREATE_NO_WINDOW`, Job Object, kill-tree, timeout | M0-3 | 5 | `src-tauri/src/win/**`, runner | no console flash; no orphan after cancel | `fake-adb` |
| M0-5 | `PlatformToolsLocator` + `adb version` parse + onboarding UI | M0-3 | 4 | locator, `Settings` | resolves SDK on a clean Win11, error card on missing | fixture |
| M0-6 | `devices -l` parser + `DeviceRegistry` + `devices://changed` | M0-3,4 | 5 | parse, state, commands | 12 fixtures pass; UI lists devices | golden |
| M0-7 | `host:track-devices` client with reconnect/backoff + 2 s poll fallback | M0-6 | 4 | host_client | plug/unplug reflected ≤500 ms; fallback verified by killing the socket | integration (fake) |
| M0-8 | Panel 8-state machine + `FeatureGrid` skeleton | M0-6 | 4 | `src/**` | every state renders with the right primary action | component |
| M0-9 | `crates/fake-adb` + fixture capture script + golden harness | M0-4 | 4 | `crates/fake-adb` | suite runs with no device on CI | CI |
| M0-10 | `.github/workflows/windows.yml` + guards + `AGENTS.md` for new dirs | all | 3 | CI, docs | green on a PR; guards fail on a seeded violation | CI |

**Exit:** panel lists real devices with hotplug, no orphan processes, CI green, no `Sources/**` change.

## M1 — Device info, appearance, input, screenshots, logcat, settings (≈40 h)

| ID | Task | h | Acceptance |
|---|---|---|---|
| M1-1 | Batched `getprop`/`wm` read → `DeviceInfo` | 4 | 1 process per refresh (assert in test), all fields shown |
| M1-2 | Feature catalog + `feature_run` + appearance features (dark, font, display, locale, TalkBack, animations) | 8 | each maps to the upstream command in `../architecture/06-adb-integration.md`; serialized; busy state |
| M1-3 | Screenshot → file + clipboard + open folder | 4 | PNG magic stripped correctly; clipboard works |
| M1-4 | Input features (back/home/recents/type/keyboard) with ASCII guard | 3 | non-ASCII rejected with an explanation |
| M1-5 | `LogcatStreamer` + coalescer + virtualized viewer + export | 8 | 20 k lines/s without jank; pause/resume; export file |
| M1-6 | Settings window (SDK, captures dir, tray/startup, theme, logs) | 5 | persists, survives restart, `settings://changed` |
| M1-7 | Tray menu + single instance + window-state | 3 | second launch focuses panel |
| M1-8 | Toast/busy/confirm components + error surface | 3 | every `ErrorCode` renders a sentence + hint |
| M1-9 | E2E harness decision (tauri-driver vs Playwright fallback), timeboxed 1 day | 8 | a window opens and a device list renders from `fake-adb` |

**Exit:** a user can connect a phone and flip appearance/input toggles, view logcat, take a screenshot.

## M2 — App actions, APK install, files, UI audit (≈38 h)

| ID | Task | h | Acceptance |
|---|---|---|---|
| M2-1 | Foreground app reader + app actions (restart/stop/clear/uninstall/reset-perms/info) | 6 | confirm on destructive; result reflected |
| M2-2 | APK picker + drag-drop + streamed install with progress | 8 | progress updates ≥2/s; all `INSTALL_FAILED_*` mapped; cancel kills the transfer |
| M2-3 | `FileTransferService`: ls/push/pull/mkdir/delete + progress + cancel | 8 | 1 GB pull cancels cleanly; traversal rejected (test) |
| M2-4 | File browser UI (virtualized, transfer queue) | 6 | 5 000 entries render ≤200 ms |
| M2-5 | `ui_dump` + uiautomator retry + TalkBack audit (port of `AccessibilityTraversal.swift:19-115`) | 8 | audit issues (no label, NAF, <48 dp, undescribed image) reproduced against fixtures |
| M2-6 | Audit results UI (list + tap-to-highlight later) | 2 | issues listed with dp values |

**Exit:** upstream P0 parity minus Layout Inspector; install and transfers are robust.

## M3 — Mirroring (**conditional**, ≈40–64 h)

Gate: ADR-0002 confirmed **and** S1–S3 passed.

| ID | Task | h | Acceptance |
|---|---|---|---|
| M3-1 | `crates/mirror`: tunnel, server push, version gate, handshake | 12 | S2 extended: 10 min stable session |
| M3-2 | Frame reader + session/media packet parsing + fixtures | 8 | bytes identical to S2 dump |
| M3-3 | Loopback server + token auth + `Origin` check | 6 | unauthorised request rejected (test) |
| M3-4 | WebCodecs renderer + frame discipline + resize/rotation | 10 | ≤120 ms glass-to-glass, no leak over 30 min |
| M3-5 | Input mapping (control channel if S2 delivered it, else `input` shell) | 8 | tap accuracy <2 device px; no stuck pointers |
| M3-6 | Fallback ladder + diagnostics overlay + settings pin | 6 | each downgrade reproducibly triggered |

## M4 — Emulator extras, wireless, multi-device (≈30 h)

AVD list/start, network/location/battery/SMS simulation, screen record, `adb pair`/`connect` + mDNS,
multi-device switcher, read-only shell console (capability-gated).

## M5 — Inspector + polish (≈40 h)

Layout Inspector (dp overlay on a frozen screenshot, design-image compare), SharedPreferences/SQLite
browsers, start-at-login, keyboard map, a11y pass, perf budgets, Mica experiment, i18n scaffold.

## M6 — Release (≈30 h)

Signing (Azure Trusted Signing), updater + keypair + `latest.json`, NSIS/MSI, clean-VM installer test,
cold-start + size measurement, `parity-checklist.md`, README/docs, 1.0 tag `win-v1.0.0`.

## Totals

| Milestone | `ESTIMATE` h |
|---|---|
| M0 | 34 |
| M1 | 40 |
| M2 | 38 |
| M3 | 40–64 |
| M4 | 30 |
| M5 | 40 |
| M6 | 30 |
| **Sum without M3** | **212 h** (~6–7 weeks solo) |
| **Sum with M3** | **252–276 h** (~7–9 weeks solo) |

## Critical path

`M0-3/4 (adbkit + process) → M0-6/7 (registry/hotplug) → M0-8 (panel) → M1-2 (features) → M1-5 (logcat)
→ M2-1/2/3 (mutating paths) → [S1–S3] → M3 → M5 (inspector) → M6`

Slack lives in M3 and M5. If the schedule slips, cut in this order: M5 inspector canvas → M4 simulation →
M3 (fall back to candidate C) → M5 app-data browsers.