# 17 — Testing strategy

Goal: **no device required for the default test run**, and no parser that ships without a recorded fixture.
Upstream has zero unit tests and one protocol smoke script (`Scripts/test-mcp.py`) — that gap is the single
biggest portability risk after the UI rewrite (`docs/architecture/12-risks-and-tech-debt.md` R2).

## 1. Seams that make testing possible

```rust
// crates/adbkit/src/runner.rs
#[async_trait]
pub trait AdbTransport: Send + Sync {
    async fn run(&self, req: AdbRequest, ct: CancellationToken) -> Result<AdbOutput, AdbError>;
    async fn stream(&self, req: AdbRequest, ct: CancellationToken) -> Result<AdbStream, AdbError>;
}

pub struct AdbRequest {
    pub serial: Option<String>,        // None only for host-level commands
    pub args: Vec<String>,             // argv, never a shell string
    pub stdin: Option<Vec<u8>>,
    pub timeout: Duration,
}
```

Two implementations: `ProcessAdbTransport` (real) and `FixtureAdbTransport` (tests). Services
(`DeviceRegistry`, `LogcatStreamer`, `PackageService`) take `Arc<dyn AdbTransport>` and are therefore
testable with zero processes.

## 2. Test pyramid

| Layer | Tool | Runs in CI | Needs device |
|---|---|---|---|
| Parser / model unit tests | `cargo test -p adbkit` | yes | no |
| Service tests with `FixtureAdbTransport` | `cargo test -p adbkit` | yes | no |
| Process-layer tests (real `fake-adb.exe`) | `cargo test -p adbkit --features process-tests` | yes | no |
| IPC contract tests (serde round-trip, error mapping) | `cargo test -p tauri-app` | yes | no |
| Frontend unit/component | Vitest + `@testing-library/svelte` | yes | no |
| Snapshot/golden UI | Vitest `toMatchFileSnapshot` | yes | no |
| E2E UI | `tauri-driver` + WebdriverIO **or** Playwright-on-`devUrl` (§5) | nightly only | no |
| Device integration | `ADB_E2E=1 OMA_SERIAL=…` | manual/nightly | **yes** |
| Performance | custom benches | nightly | no (synthetic) |

## 3. Golden fixtures (the core discipline)

`crates/adbkit/tests/fixtures/<parser>/<case>.{in,out.json}` — the `.in` file is **verbatim** captured
output; the `.out.json` is the expected model.

| Parser | Cases required |
|---|---|
| `adb devices -l` | one device, multiple, unauthorized, offline, no-permissions, emulator, `bootloader`, empty, CRLF, trailing spaces, missing `model:` |
| `adb version` | 1.0.41, 1.0.39 (unsupported), git-describe string |
| `getprop` batch | normal, missing property (empty line), value with spaces, non-UTF-8 bytes |
| `wm size` / `wm density` | with `Physical size` + `Override size`, density only, malformed |
| `df` | `tmpfs` vs `f2fs`, missing mount |
| `pm list packages -3` | normal, empty, package with dots/digits |
| `dumpsys activity activities` | single resumed, split-screen (two resumed), none |
| `uiautomator dump` XML | nested, null labels, duplicate ids, `bounds` edge cases |
| `logcat -v threadtime` | standard, multi-line stack traces, `beginning of` marker, buffer-cleared marker |
| `adb push` / `pull` stderr | progress lines, final summary, `failed to copy` |
| `install` output | `Success`, `INSTALL_FAILED_*` (≥6 variants), `Performing Streamed Install` prefix |
| `adb connect`/`pair` | success, `failed to connect`, wrong pairing code |
| scrcpy headers | codec id, session packet, config+keyframe media packet, PTS overflow |

Fixture rules:

1. Never hand-write a fixture from memory — capture it (`Scripts/capture-fixture.sh` in M0) and redact
   serials/model names.
2. Every fixture has a comment with the device/OS/adb version it came from.
3. A parser change that alters a golden output is a **reviewed diff**, never a blind re-bless. Re-bless via
   `UPDATE_GOLDEN=1 cargo test`, which prints the diff.

## 4. Fake adb

Decision: **a real executable** (`crates/fake-adb`, `[[bin]] name = "fake-adb"`), not just in-process fakes.

Why: the riskiest code is the *process* layer (CREATE_NO_WINDOW, Job Objects, cancellation kill, CRLF,
encoding). In-process doubles cannot test that. The fake reads a scenario JSON from `%FAKE_ADB_SCENARIO%`
and can: sleep, emit stderr progress, exit with a code, emit CRLF, emit invalid UTF-8, spawn a grandchild
(to prove Job Object kill-tree), and close stdout early.

`process-tests` feature puts `fake-adb.exe`'s directory first on `PATH` and requests `fake-adb` as the
binary name.

## 5. UI tests

| Kind | Tool | Notes |
|---|---|---|
| Unit (pure fns: `panelState`, formatting, error mapping) | Vitest | fast, most value |
| Component | Vitest + `@testing-library/svelte` | render `Panel` with a mocked `ipc.ts` |
| Golden markup | Vitest file snapshots | catches unintended visual/structural churn |
| E2E | **first choice:** `tauri-driver` + WebdriverIO on Windows; **fallback:** Playwright against `npm run dev` (`devUrl`) with `ipc.ts` mocked | Both `UNVERIFIED`. The WebDriver path is known to be finicky on Windows (`tauri-driver` requires `msedgedriver` matching the installed WebView2); timebox it to 1 day in M1 and drop it for the Playwright fallback if it fights back |

Accessibility: `axe-core` in component tests for the panel and settings; keyboard-only smoke test is manual
in M5.

## 6. Performance tests (all synthetic, no device)

| Test | Pass |
|---|---|
| Logcat flood: 20 000 lines/s for 60 s through the coalescer into a mock UI consumer | UI frame budget <16 ms p95, memory growth <50 MB, dropped counter exposed |
| Device list: 50 devices, 10 hotplug events/s | `devices://changed` ≤5/s after debounce |
| File list: 5 000 entries | first render ≤200 ms, scroll 60 fps |
| Screenshot pipeline: 3 MB PNG → disk → asset URL | ≤150 ms |
| Mirror parse (offline, synthetic stream): 8 Mbps Annex-B | parse CPU <5 % of one core, no allocation growth over 10 min |
| Cold start | ≤1.5 s to interactive panel |

Mirror-specific memory test is the important one: a leaked `VideoFrame` is invisible in dev and fatal in
production (`13-mirroring-design.md` §2.4).

## 7. Device integration tests

Phase-gated, read-only by default (`docs/CONTRIBUTING-fork.md` ADB safety rules):

```sh
ADB_E2E=1 OMA_SERIAL=R58M1234ABC cargo test -p adbkit --features e2e -- --nocapture
```

Allowed without extra opt-in: `devices -l`, `get-state`, `getprop`, `pm list packages`, `wm size`,
`wm density`, `screencap`, read-only `dumpsys`, `logcat -d`.
Requires `ADB_E2E_MUTATE=1` **and** an explicit serial: anything else.

Tests must skip (not fail) when the env vars are absent, and report a clear skip reason.

## 8. CI gates (blocking)

1. `cargo fmt --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace` (fixtures + fake-adb)
4. `npm run check && npm run test`
5. `npm run check:bindings` (no drift between Rust and `bindings.ts`)
6. `rg` guards: no `Command::new` outside `adbkit`/`win`, no `adb` literal in `src/`, no `{@html}`, no
   network client outside the updater
7. `npm run tauri build` succeeds (bundler regressions caught on every PR)

Non-blocking/nightly: E2E, perf benches, device matrix, installer smoke on a clean VM.

## 9. What upstream gives us for free

| Upstream artefact | Use |
|---|---|
| `Scripts/test-mcp.py` | not ported (no MCP), but its assertions document the 18 tool behaviours — read it when writing `PackageService`/`InputService` tests (`Sources/MCP/Tools/*.swift`) |
| Adb command inventory | `docs/architecture/06-adb-integration.md` is the test-case list for the fixture suite |
| 8-state panel model | `docs/architecture/08-ui-ux-map.md` §3 is the component test matrix |