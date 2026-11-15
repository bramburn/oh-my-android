# ADR-0002 — Screen mirroring

- Status: **Proposed — blocking question for Gate 3**
- Date: 2026-11-15
- Deciders: bramburn (product owner)
- Supersedes: the Gate 1 note in `docs/architecture/07-mirroring-and-control.md` ("no mirroring")

## Context

**The upstream product has no mirroring.** `rg -ni "scrcpy|app_process|localabstract|h264|MediaCodec"
Sources` returns one hit: the word "mirrored" inside an LLM prompt string
(`Sources/MCP/Server/Catalog.swift:40`). Everything visual upstream is *snapshot* based:

- `adb exec-out screencap -p` → PNG bytes (`Sources/Core/Android/ADBClient.swift:21-27`);
- `uiautomator dump` → XML tree → dp measurements (`Sources/Core/Android/LayoutSnapshot.swift:82-109`);
- input injected with `adb shell input …` (`Sources/Features/InputFeatures.swift`).

So mirroring is a **new feature**, not a port, and it is the only feature in this plan whose feasibility
is unknown before building it.

Two decisions conflict:

| Source | Statement |
|---|---|
| Gate 1 (this session) | "no live screen mirroring" |
| Gate 3 brief | video is "one hard problem"; spike candidates A–D before planning |

This ADR exists so the conflict is resolved explicitly rather than by drift.

## Options

| # | Option | Effort `ESTIMATE` | Latency `ESTIMATE` | Risk | Notes |
|---|---|---|---|---|---|
| A | Own scrcpy client in Rust (`crates/mirror`): push `scrcpy-server.jar`, `app_process`, `adb forward`, parse frames, ship Annex-B to WebCodecs | 8–12 d | 35–70 ms | High: protocol is **explicitly unstable** between scrcpy versions (quoted below) | Best result; must be pinned to one server version and version-gated |
| B | `scrcpy.exe` as a frame pump: `scrcpy --no-playback --no-control --record=pipe`→ parse H.264, no protocol code | 3–5 d | 60–120 ms | Medium: file/pipe parsing, disk hygiene if a file is used, scrcpy `.exe` must be bundled (GPL? see below) | De-risks A |
| C | Sibling `scrcpy.exe` window; our app is only the control panel | 0.5–1 d | n/a (separate window) | Low | Ships a day; not an integrated product, and adds a 2nd window to manage |
| D | `screencap` polling at 2–5 fps + `input` events | 0.5 d | ~300 ms | Low | Universal fallback; already matches upstream behaviour |

**Rejected up front:** embedding `scrcpy.exe`'s HWND with `SetParent` into the webview. Airspace/clipping/
DPI/focus problems are structural in WebView2 and would cost a week with no clean exit.

### The protocol-stability problem (why A is risky)

scrcpy's own developer documentation, at the pinned tag `v5.0.1`
(`https://raw.githubusercontent.com/Genymobile/scrcpy/v5.0.1/doc/develop.md`):

> "The protocol between the client and the server must be considered *internal*: it may (and will) change
> at any time for any reason. Everything may change (the number of sockets, the order in which the sockets
> must be opened, the data format on the wire…) from version to version. A client must always be run with a
> matching server version."

And that same file states "This section documents the current protocol in scrcpy 4.0" while the tag is
`v5.0.1` — i.e. the prose itself lags the code. Any implementation of A must therefore derive the wire
format from the **pinned server jar's own source**, not from the prose (`13-mirroring-design.md` §3).

### Licensing note for B/C

`scrcpy` is Apache-2.0 (client + server). Bundling `scrcpy.exe` is a distribution decision that must be
recorded in `NOTICE.md` with its NOTICE file. `ESTIMATE`: adds ~6 MB to the installer.

## Decision (proposed)

1. **M0–M2 do not depend on mirroring.** Ship snapshot + input injection (option D) as the baseline in
   M1, exactly as upstream behaves.
2. Run spikes **S1 (transport), S2 (handshake), S3 (WebCodecs)** before any M3 commitment.
3. If S2 **and** S3 pass: implement **A**. If S3 fails but S2 passes: implement **B** with a native
   render path or a sibling window. If both fail: **C**, and keep D inside the app.
4. **D always ships**, because it is upstream parity and the fallback for every device where video cannot
   start (locked screen, `secure` flag, low-end codec).

## Consequences

- The architecture in `11-tauri-architecture.md` keeps video behind a `MirrorSession` trait so A/B/C/D are
  interchangeable at the boundary.
- `13-mirroring-design.md` is written as *Proposed*; nothing else in this plan cites it.
- If the product owner re-confirms "no mirroring", M3 becomes "input injection + snapshot polish" and the
  docs shrink by ~40 % with no architectural change.

## Blocking question

Is live interaction in scope for v1 — i.e. can the user *tap and type into a live view*, or is
snapshot + scripted input + the Layout Inspector enough? Answer at Gate 3; this ADR stays `Proposed`
until then.

## Evidence

- `docs/architecture/07-mirroring-and-control.md` — full absence analysis with commands and line refs.
- scrcpy protocol prose: `https://raw.githubusercontent.com/Genymobile/scrcpy/v5.0.1/doc/develop.md`
  (retrieved 2026-11-15; quotes above are verbatim).
- scrcpy latest release `v5.0.1`, published 2026-10-08 — `https://api.github.com/repos/Genymobile/scrcpy/releases/latest`.
- Tauri custom-protocol bodies are buffered (`http::Response<Vec<u8>>`), so video must not ride on it:
  `https://docs.rs/tauri/2.12.5/tauri/struct.Builder.html#method.register_asynchronous_uri_scheme_protocol`.