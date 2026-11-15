# 19 — Risk register

Scored for the Windows port. `L` = likelihood, `I` = impact, both 1–5. Trigger = the observable that means
the risk has materialised.

| # | Risk | L | I | Trigger | Mitigation | Fallback |
|---|---|---|---|---|---|---|
| R1 | **WebCodecs H.264 unavailable/broken in WebView2** | 3 | 4 | S3 fails or decode is software-only | S3 in week 1; `isConfigSupported` gate at startup | candidate B (scrcpy pump, native child window) or C (sibling window) |
| R2 | Mirroring protocol drift / handshake bugs | 4 | 3 | garbage codec id, session dies | pin scrcpy server version, version-gate the handshake, golden byte fixtures | rung D (screencap + input) |
| R3 | **Orphan `adb.exe` processes / console flashes** | 3 | 4 | `tasklist` shows adb after exit | Job Object + `CREATE_NO_WINDOW` + S4 test on every CI run | none needed; this is testable, so it will be tested |
| R4 | Parser fragility across OEM/Android versions | 4 | 3 | `PARSE_ERROR` in the wild | fixture suite per parser + `PARSE_ERROR` telemetry-free diagnostics the user can copy | degrade the single feature, keep the panel alive |
| R5 | Scope creep from the Layout Inspector | 4 | 3 | M5 blocks the release | it is P1, not P0; ship 1.0 without it | drop from 1.0 |
| R6 | WebView2 missing/old on target machines | 2 | 3 | blank window on a clean VM | `embedBootstrapper` + S5 clean-VM test | `offlineInstaller` (+130 MB) or `fixedRuntime` |
| R7 | Two-language codebase degrades (Rust/TS drift) | 3 | 3 | `bindings.ts` drift, duplicated logic | generated bindings + CI drift check + "thin commands" rule | contract tests per command |
| R8 | Updater supply-chain compromise | 1 | 5 | unexpected signature/key | new keypair in CI secret only, opt-in updates, HTTPS endpoint owned by the fork | disable updates (build flag) |
| R9 | Command injection via device-controlled strings | 2 | 5 | a `'` in a package name breaks a command | single quoting helper + argv for `adb.exe` + injection unit tests | — |
| R10 | Installer size creep past 25 MB | 3 | 2 | CI artifact report | budget table in `16-`; lazy-download extras | ship "bring your own SDK" variant only |
| R11 | Latency budget missed (>120 ms) | 3 | 3 | S3 measurement | `optimizeForLatency`, hardware decode, `OffscreenCanvas`, drop policy | fall back to B/C |
| R12 | Logcat volume freezes the UI | 3 | 3 | dropped counter spikes, frame time >16 ms | bounded channel + drop-oldest + virtualization + perf test | lower default buffer to 10 k lines |
| R13 | macOS CI rot breaks the untouched workflow | 2 | 2 | `build.yml` red | it is untouched; if it breaks, fix minimally in its own commit (never delete) | pin runner/macOS image in that workflow only |
| R14 | Windows-specific encoding bugs (CRLF, OEM codepage) | 3 | 2 | fields with `\r`, mojibake model names | fixture suite includes CRLF and invalid UTF-8 | lossy decode everywhere |
| R15 | Multi-device race (wrong device mutated) | 2 | 5 | action applied to a non-active device | serial mandatory in every request type; no implicit-device API exists | compile-time: `AdbRequest.serial` is `Option` only for host commands, with a clippy lint |
| R16 | Long-path/filename failures in transfer | 3 | 2 | `ERROR_PATH_NOT_FOUND` on deep trees | `\\?\` prefix, reserved-name rejection | fall back to a user-chosen shallow dir |
| R17 | Signing not obtained before release | 4 | 2 | SmartScreen wall on first download | decide by M4, budget ~$10/mo | ship unsigned with a prominent warning + SHA-256 published |
| R18 | Keyboard/focus fight with the emulator (no-activate panel) | 3 | 3 | clicks steal focus, typing goes nowhere | `WS_EX_NOACTIVATE` + explicit "pin focus" toggle | plain always-on-top window |
| R19 | Unverified toolchain versions (TS 7, Vite 8) break the build | 3 | 2 | `npm run build` fails at M0 | M0-1 resolves and records the actual working set | downgrade pin in `10-version-pin.md` |
| R20 | Docs drift from implementation | 3 | 3 | AGENTS.md contradicts code | doc-drift rule + PR checklist | review |

## Top 5 (must be resolved before M3)

1. **R1** — WebCodecs in WebView2 (S3). Decides whether mirroring is possible at all.
2. **R3** — process lifecycle (S4). Non-negotiable quality bar for a panel that runs constantly.
3. **R4** — parser fragility. The reason upstream has no tests and a rewrite usually regresses here.
4. **R2** — protocol stability. Only matters if R1 passes, but it is then the largest unknown.
5. **R15** — wrong-device mutation. The only "could destroy someone's data" risk in the product.

## Risk review cadence

Re-score at the end of each milestone; a risk whose trigger fires gets an ADR (not a silent workaround), and
a risk that stops being relevant moves to a "closed" table with the reason.