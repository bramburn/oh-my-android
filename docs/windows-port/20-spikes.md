# 20 — Spikes

All throwaway code lives in `spikes/<id>/`, is excluded from the workspace (`exclude = ["spikes"]` in the
root `Cargo.toml`), and is **deleted** after its verdict — the numbers below are the deliverable, not the
code. Each spike has a README with run instructions, raw measurements, and a verdict.

**None of these have been run.** This machine has no Tauri toolchain installed, no device, and `adb` is not
on `PATH`; S1–S3 also require WebView2. Everything below is the plan for running them, not a result.

## S1 — IPC throughput for streaming bytes (≤4 h)

| | |
|---|---|
| Question | Which Rust→UI transport sustains 8 Mbps video-ish payload plus a logcat flood? |
| Setup | Tauri app with 4 command variants emitting the same synthetic 8 Mbps Annex-B byte stream: (a) `Channel<Vec<u8>>`, (b) `Channel<InvokeResponseBody>`, (c) loopback HTTP + `ReadableStream`, (d) custom protocol; plus a logcat variant at 20 k lines/s |
| Measure | MB/s, CPU % (Task Manager + `GetProcessTimes`), UI frame time (PerformanceObserver long tasks), memory drift over 10 min |
| Pass | One transport ≥8 Mbps with **<5 % CPU** and **no long task >50 ms** |
| Fail ⇒ | Linux-style conclusion: video goes native (child window) |
| Numbers | *pending* |

## S2 — scrcpy handshake + frame parsing (≤8 h)

| | |
|---|---|
| Question | Can we drive the pinned server and parse its stream correctly? |
| Setup | Push `scrcpy-server.jar` (v5.0.1), `adb reverse`, `app_process`, connect, dump the first 256 bytes of the video socket to a file, then decode headers for 30 s |
| Measure | Dummy byte present (forward mode), device-name metadata, codec id, session packet count on rotation, media packets/s, keyframe cadence, PTS continuity |
| Pass | ≥10 s of parseable H.264 with **SPS/PPS identified** and **keyframes flagged**; control-channel message layout documented from the pinned source |
| Fail ⇒ | candidate B (`scrcpy.exe` pump) or C |
| Numbers | *pending* |

## S3 — WebCodecs in WebView2 (≤6 h)

| | |
|---|---|
| Question | Does WebView2 decode H.264 Annex-B via `VideoDecoder` with hardware acceleration at low latency? |
| Setup | Minimal page: `VideoDecoder.isConfigSupported({codec:'avc1.42E01E', avc:{format:'annexb'}, hardwareAcceleration:'prefer-hardware', optimizeForLatency:true})`, feed the S2 dump, draw to canvas, add a millisecond clock on the device screen for glass-to-glass |
| Measure | `isConfigSupported` result, `hardwareAcceleration` effective (chrome://gpu inside WebView2 / `VideoDecoder` stats), glass-to-glass ms (phone camera at 240 fps), frame loss over 5 min, `decodeQueueSize` behaviour |
| Pass | supported = true, hardware decode active, **glass-to-glass < 120 ms**, **no frame leak over 5 min** |
| Fail ⇒ | ADR-0002 rung B/C; if WebCodecs is absent entirely, drop integrated video from v1 |
| Numbers | *pending* |

## S4 — Process lifecycle (≤2 h)

| | |
|---|---|
| Question | Do `CREATE_NO_WINDOW` + Job Object reliably kill whole trees with no flash and no orphans? |
| Setup | `fake-adb` spawning a grandchild; spawn 200 times in a loop; cancel mid-flight; kill the parent app via Task Manager |
| Measure | Console flash count (visual + `GetConsoleWindow` check), survivors in `tasklist` after each phase, time from cancel to gone |
| Pass | 0 flashes, **0 orphans**, cancel-to-gone **≤1 s** |
| Fail ⇒ | `CREATE_SUSPENDED` + assign + resume; if still failing, a watchdog `taskkill` sweep on exit |
| Numbers | *pending* |

## S5 — WebView2 presence/versions (≤2 h)

| | |
|---|---|
| Question | What does a clean Win10 22H2 / Win11 have, and does `embedBootstrapper` behave offline? |
| Setup | Fresh VMs: Win10 22H2 (no updates), Win11 24H2. Install the NSIS build offline. Record
`HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}` version |
| Pass | App starts on both; installer works with no network (or a clear, actionable message) |
| Fail ⇒ | `offlineInstaller` (+~130 MB) or `fixedRuntime` |
| Numbers | *pending* |

## S6 — Bundle size + cold start (≤3 h)

| | |
|---|---|
| Question | Does the installer fit ≤25 MB and start ≤1.5 s? |
| Setup | `npm run tauri build` with/without `platform-tools` and `scrcpy`; measure cold start with a stopwatch from process start to panel interactive (log a timestamp in Rust at first paint) |
| Pass | ≤25 MB and ≤1.5 s |
| Fail ⇒ | move `platform-tools` to bring-your-own (already the default), lazy-load logcat window |
| Numbers | *pending* |

## Decision table (fill on completion)

| Spike | Verdict | Key numbers | Plan change |
|---|---|---|---|
| S1 | | | transport |
| S2 | | | A vs B |
| S3 | | | integrated video yes/no |
| S4 | | | process layer done/needs suspended-spawn |
| S5 | | | WebView2 install mode |
| S6 | | | packaging |

## Ordering rule

**Run S1–S4 before M3 is scheduled, and S3 before the mirroring section of any plan is treated as
committed.** S4 can run inside M0-4 (it is the same code), S5/S6 inside M0-2 and M6 respectively.