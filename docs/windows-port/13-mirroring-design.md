# 13 — Mirroring design (**Proposed**, ADR-0002)

Do not build M3 from this document until S1–S3 pass. Nothing in M0–M2 depends on it.

## 1. What upstream does (baseline that always ships)

| Capability | Upstream implementation | Latency |
|---|---|---|
| See the screen | `adb exec-out screencap -p` → PNG (`Sources/Core/Android/ADBClient.swift:21-27`) | ~150–400 ms per frame `ESTIMATE` |
| Understand the screen | `uiautomator dump` → XML → dp bounds (`Sources/Core/Android/LayoutSnapshot.swift:82-109`) | 300–900 ms |
| Interact | `adb shell input tap/swipe/text/keyevent` (`Sources/Features/InputFeatures.swift`, `Sources/MCP/Tools/InputTools.swift`) | 30–80 ms per event |

That combination is **already parity** and is the fallback for every failure mode below.

## 2. Candidate A — own scrcpy client in Rust (`crates/mirror`)

### 2.1 Session setup

From scrcpy v5.0.1 `doc/develop.md` §Protocol/Connection
(`https://raw.githubusercontent.com/Genymobile/scrcpy/v5.0.1/doc/develop.md`):

```
1. adb push <scrcpy-server.jar> /data/local/tmp/scrcpy-server.jar
2. tunnel (pick one):
   adb reverse localabstract:scrcpy_<SCID> tcp:27183      # default: host listens, device connects
   adb forward tcp:27183 localabstract:scrcpy_<SCID>      # fallback: device listens, host connects
3. adb shell CLASSPATH=/data/local/tmp/scrcpy-server.jar app_process / com.genymobile.scrcpy.Server <version> \
       scid=<SCID> log_level=info audio=false max_size=1920 ...
4. open sockets in this order: video, audio, control  (each may be disabled by a flag)
5. on the FIRST socket: if the tunnel is forward, the device sends 1 dummy byte;
   then device metadata (currently just the device name: name length + UTF-8 bytes)
```

`<SCID>` is a 31-bit random number so concurrent sessions on one device do not collide. The `app_process`
argument must match the **exact bundled server version** — the doc's own example still says `4.0` while the
tag is `v5.0.1`, which is precisely why the version gate (below) is mandatory rather than cosmetic.

### 2.2 Video stream wire format

Both facts below are quoted from the same document; the field layout is not guesswork.

- First `u32` on the video socket is the **codec id**: `"h264"` = `0x68323634`, `"h265"` = `0x68323635`,
  `"av1"` = `0x00617631`, `"vp8"` = `0x00767038`, `"vp9"` = `0x00767039`.
- **Session packet, 12 bytes, one per capture session** (a new session starts on rotation):
  `byte0 = 0b1000_0000` (session flag), `byte3 bit0` = *client resized* flag, `bytes 4..8` = width (u32 BE),
  `bytes 8..12` = height (u32 BE).
- **Media packet, 12-byte header + payload**:
  byte0 bit7 = media flag, bit6 = config packet (`SPS/PPS`), bit5 = key frame; low 5 bits of byte0 + bytes
  1..7 = PTS (61 bits); bytes 8..12 = packet size (u32 BE).

```
byte 0    1 2 3 4 5 6 7   8 9 10 11
0CK.....  <----- PTS ----> <-- size -->
 ^^^
 ||`- key frame
 |`-- config packet
 `--- media packet
```

Client obligations that follow from this layout:

1. Feed the decoder **config packets before the first key frame** (Annex-B with SPS/PPS inline).
2. Treat a session packet as a **decoder reconfigure + canvas resize**, not a repaint.
3. On rotation, wait for the next config+key frame before resuming; never feed stale PTS.
4. `SPIKE S2` must confirm the *actual* layout by dumping the first 256 bytes and decoding them, because
   prose in this project lags its code.

### 2.3 Control channel

Encodings for pointer/key/clipboard messages are **`UNKNOWN — SPIKE S2`**. They must be derived from the
pinned server source (`server/src/main/java/com/genymobile/scrcpy/control/ControlMessage.java` at tag
`v5.0.1`), not from memory. Until S2 delivers them, input goes through `adb shell input` (§5).

### 2.4 Failure modes

| Mode | Signal | Handling |
|---|---|---|
| `FLAG_SECURE` window (banking apps) | decoder produces black frames / server errors | detect all-black frames for >2 s → `MIRROR_SECURE_DEVICE` → fall back to D with an explanation |
| Screen off | no frames | show "screen off" state, offer `input keyevent 26` |
| Server version mismatch | handshake fails / garbage codec id | kill session, re-push jar, one retry, then error |
| Device rotated | session packet | reconfigure, do not restart the session |
| USB unplug | socket EOF | tear down, state `DEVICE_DISCONNECTED` |
| Decoder queue stall | `decodeQueueSize` > 8 | drop non-key frames, then `reset()` the decoder |
| Low-end device / encoder busy | fps < 15 for 3 s | downgrade `max_size` 1920→1280→1024 |

## 3. Candidate B — `scrcpy.exe` as a frame pump

`scrcpy --no-playback --no-control --no-audio --record=- --record-format=h264` writes an elementary Annex-B
stream. We parse it with the **same** `crates/mirror` frame reader, minus the handshake. Cheaper than A by
~5 days, at the cost of: a bundled `scrcpy.exe` (+~6 MB, Apache-2.0, NOTICE entry), no control channel
(so input still goes through `adb shell input`), and process-lifecycle coupling. `SPIKE S1`-adjacent risk:
whether `--record=-` streams or buffers. **`UNVERIFIED`** — verify at S2 by timing the first bytes.

## 4. Transport: loopback HTTP, not IPC

| Option | Streaming? | Verdict |
|---|---|---|
| `Channel<InvokeResponseBody>` | yes, but one JSON/IPC frame per chunk + main-thread hop | works, measurably worse at 8 Mbps |
| `Channel<Vec<u8>>` | **no** — serialises bytes as a JSON array | forbidden |
| Custom protocol (`register_asynchronous_uri_scheme_protocol`) | **no** — responder takes `http::Response<Vec<u8>>`, i.e. a fully materialised body (`VERIFIED`, docs.rs 2.12.5) | rejected |
| **`127.0.0.1:<ephemeral>` axum route + `fetch` + `ReadableStream`** | yes | **chosen** |

Server hardening (non-negotiable):

- bind `127.0.0.1` only; **ephemeral port** (0), reported to the UI in the `mirror_start` response;
- random 128-bit **per-session token** in the path or `Authorization` header, compared in constant time;
- reject requests whose `Origin` is not the app's origin (`http://tauri.localhost` or the configured dev
  origin) — cheap CSRF defence against a browser on the same machine;
- one route, lifespan = session lifespan; server shut down on session end;
- CSP must allow exactly this origin (`15-plugin-and-security.md` §3).

## 5. Renderer and input

| Concern | Decision |
|---|---|
| Decoding | WebCodecs `VideoDecoder`, `avc: { format: 'annexb' }`, `optimizeForLatency: true`, `hardwareAcceleration: 'prefer-hardware'` → **`SPIKE S3`** (WebView2 support unverified) |
| Rendering | `drawImage(VideoFrame)` on a `<canvas>` sized by `devicePixelRatio`; `OffscreenCanvas` in a Worker only if S3 shows main-thread jank |
| Frame discipline | always `frame.close()`; cap `decodeQueueSize` at 4; `reset()` on reconfigure |
| Scaling | letterbox to aspect ratio; integer scaling option; nearest-neighbour for pixel-peeping |
| Input mapping | pointer coords → normalised → device px via the session packet's width/height; clamp to bounds |
| Input transport | control channel if S2 delivers it; otherwise `adb shell input tap/swipe` (adds ~40 ms `ESTIMATE`, no multitouch) |
| Keyboard/text | IME text via `input text` with the ASCII guard + `%s` space escape (upstream rule, `Sources/Features/InputFeatures.swift:25-43`) |
| Clipboard | device→host only in MVP (`clipper` broadcasts are fragile); host→device via `input text` is a `SPIKE` |
| Audio | **deferred.** Requires an extra socket + an audio decoder + ~1 week. Not in v1. |
| Multi-touch | only with the control channel; otherwise single pointer |

## 6. Latency budget (target ≤120 ms glass-to-glass, `ESTIMATE` per hop)

| Hop | Budget | Confidence |
|---|---|---|
| Device capture (MediaCodec `surface`) | 16–33 ms | measured by scrcpy's own design (`develop.md` §Screen video encoding) |
| Encode (H.264, hardware) | 10–20 ms | `ESTIMATE` |
| USB/adb transport | 5–15 ms | `ESTIMATE` |
| Rust parse + loopback HTTP write | 1–3 ms | `ESTIMATE` |
| WebView2 `fetch` + `ReadableStream` | 2–6 ms | `ESTIMATE` |
| WebCodecs decode (hardware) | 5–15 ms | depends on S3 |
| Canvas present | 8–16 ms (one vsync) | physics |
| **Total** | **47–108 ms** | S3 must measure end-to-end with a moving clock on screen |

Fail thresholds: >120 ms ⇒ try `OffscreenCanvas`/native child window; >200 ms ⇒ fall back to B.

## 7. Fallback ladder (automatic, with a visible reason)

```
A (own client, control channel)
  └─ handshake/codec failure ──► B (scrcpy.exe pump, no control)
        └─ decode failure in WebView2 ──► C (sibling scrcpy window)
              └─ scrcpy unavailable ──► D (screencap polling + input shell)
```

Every downgrade emits a toast with the reason and a "why?" link to a docs anchor. The user can pin a lower
rung in settings (useful for debugging). Metric that triggers each downgrade is in §2.4.

## 8. Work estimate `ESTIMATE`

| Item | Days |
|---|---|
| S1–S3 spikes | 2 |
| Protocol client + tunnel + lifecycle (A) | 5 |
| Frame reader + fixtures | 2 |
| Loopback server + renderer + input mapping | 3 |
| Fallback ladder + diagnostics | 2 |
| B (if chosen instead of A) | 3 |
| **Total for M3** | **~7–12** |

## 9. Open questions

1. Is live interaction required for v1? (ADR-0002 blocking question.)
2. Bundle `scrcpy.exe`/jar, or require the user to supply them like `platform-tools`? (Consistency argues
   for "bring your own"; UX argues for bundling.)
3. Is audio a v1 requirement? Currently deferred.