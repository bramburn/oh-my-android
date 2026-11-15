# 15 — Plugins, capabilities and security

## 1. Plugin table

| Plugin | Version | Purpose | Permissions granted | Risk | Alternative |
|---|---|---|---|---|---|
| `tauri-plugin-log` | 2.10.2 | file + stdout logs | `log:default` (write own log dir) | low | hand-rolled `tracing` appender |
| `tauri-plugin-store` | 2.5.1 | settings persistence | `store:default` | low | own JSON file in `%APPDATA%` — **preferred**, since settings are typed and small; keep store only if it earns its keep |
| `tauri-plugin-dialog` | 2.8.4 | native open/save | `dialog:default` | low | Win32 `IFileOpenDialog` |
| `tauri-plugin-fs` | 2.7.1 | **scoped** reads for the design-overlay image + captures folder reveal | `fs:allow-read` limited to `$APPDATA/captures` and user-picked paths via `dialog` scope | medium — scope bugs = arbitrary file read | none needed if all file IO stays in Rust |
| `tauri-plugin-single-instance` | 2.5.5 | one instance, second launch focuses panel | `default` | low | named mutex |
| `tauri-plugin-window-state` | 2.5.0 | remember panel/window geometry | `default` | low | own settings keys |
| `tauri-plugin-clipboard-manager` | 2.5.0 | screenshot → clipboard | `clipboard-manager:allow-write-image` only (**not** read) | low | `arboard` in Rust |
| `tauri-plugin-opener` | 2.7.3 | "open captures folder", open docs URL | `opener:allow-open-path` restricted to the captures dir + `allow-open-url` for one docs host | medium (URL handler abuse) | `ShellExecuteW` with a fixed allowlist |
| `tauri-plugin-os` | 2.4.0 | platform/arch for diagnostics | `default` | low | `cfg!(windows)` |
| `tauri-plugin-notification` | 2.8.1 | install finished / device lost when panel hidden | `notification:default` | low | native toast via `windows` crate |
| `tauri-plugin-updater` | 2.14.1 | opt-in updates | `updater:default` | **high** — arbitrary code execution if the key/feed is compromised | none; must be done correctly (§5) |
| ~~`tauri-plugin-shell`~~ | — | — | — | **rejected** | see §2 |
| `tauri-plugin-process` | — | restart app after update | `process:allow-restart` only | low | `std::process::exit` + relaunch |

## 2. Why `tauri-plugin-shell` is rejected for adb

1. It exposes process spawning to **frontend JS**, creating a second, unaudited adb path that bypasses
   `crates/adbkit` — the single-choke-point invariant (`11-tauri-architecture.md` §2) dies immediately.
2. Its scoping model is command+args allowlists; adb needs per-device serials, dynamic paths and
   stream parsing, none of which it models.
3. It offers no Job Object control, no `CREATE_NO_WINDOW` guarantee, no bounded output backpressure — the
   three things that make a multithreaded adb GUI stable on Windows.
4. It is the single largest injection surface in a Tauri app: device-controlled strings (package names,
   file paths, `dumpsys` output) must never reach a shell-command constructor. In Rust they reach an
   argv array instead.

## 3. Capabilities (least privilege, per window)

`withGlobalTauri: false`; capabilities listed explicitly in `app.security.capabilities` so nothing is
auto-enabled (default behaviour, `VERIFIED`: https://tauri.app/security/capabilities/ — "All capabilities
inside the capabilities directory are automatically enabled by default. Once capabilities are explicitly
enabled in tauri.conf.json, only these are used").

```json
// src-tauri/capabilities/panel.json
{ "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "panel",
  "description": "Floating panel: read devices, run features, capture screenshots",
  "windows": ["panel"],
  "permissions": ["core:default", "core:event:default", "log:default",
                  "clipboard-manager:allow-write-image", "window-state:default"] }
```

```json
// src-tauri/capabilities/logcat.json
{ "identifier": "logcat",
  "windows": ["logcat"],
  "permissions": ["core:default", "core:event:default", "dialog:allow-save", "fs:allow-write-text-file"] }
```

`settings.json` gets `dialog:allow-open`, `autostart` only if that plugin is added; `updater` permission is
scoped to the settings window only so a compromised panel cannot trigger an update check.

**App-defined commands** (our `#[tauri::command]`s) are *not* gated by capabilities the way plugin commands
are — they are reachable by any window that can `invoke`. Therefore every app command still validates its
own preconditions server-side (serial present, action allowed, path inside an allowed root). Capabilities
protect the plugin surface; the Rust handlers protect ours. This distinction is easy to get wrong and is a
review checklist item.

## 4. CSP

```json
"security": {
  "csp": "default-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
          img-src 'self' data: blob: asset: http://asset.localhost; \
          media-src blob:; font-src 'self'; \
          connect-src 'self' ipc: http://ipc.localhost http://127.0.0.1:*; \
          frame-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'"
}
```

| Directive | Why |
|---|---|
| `connect-src … http://127.0.0.1:*` | the loopback video server (port is ephemeral, so a wildcard port is required) |
| `ipc:` / `http://ipc.localhost` | Tauri's IPC on Windows |
| `'unsafe-inline'` for styles | Tailwind v4 + Svelte inject styles; alternatively hash them in M5 |
| `blob:` for `media-src` | `MediaSource`/`VideoFrame` blobs |
| no remote origins anywhere | no CDN, no remote fonts, no analytics |

`devUrl` origin is added only in dev builds. Devtools disabled in release (`"devtools": false` on the
window config, `tauri.conf.json`).

## 5. Updater

- New minisign keypair; public key in `tauri.conf.json > plugins.updater.pubkey`; private key only in CI
  secrets. **Never** upstream's `SUPublicEDKey` (`project.yml:58`).
- Endpoint: the fork's own `latest.json` on GitHub Releases.
- Update check is **opt-in** (off by default), matching the "no telemetry" posture: an unconditional
  startup GET is a beacon.
- `dangerousInsecureTransportProtocol` stays false; HTTPS only.
- Rollback: keep the previous installer for one version; `allowDowngrades` stays `true` in dev, `false` in
  release.

## 6. Threat model

| Threat | Vector | Mitigation |
|---|---|---|
| Command injection into device shell | package names, file paths, `getprop` values, SMS text | everything reaching `adb shell` is single-quoted via `shell_quote.rs`; argv (never a shell) at the `adb.exe` level; a unit test asserts `'` escaping |
| Path traversal writing to host | device-supplied file names in `pull` | resolve + canonicalize + assert the result stays inside the chosen root; reject `..`, drive letters, reserved names (`CON`, `PRN`, `AUX`, `NUL`, `COM1`…) |
| Path traversal writing to device | user-supplied remote path in `push` | normalize, reject `..` components, require an absolute path |
| Malicious/oversized device output | 100 MB `dumpsys`, binary garbage | bounded readers (cap 8 MB per one-shot, 1 MB per line), lossy UTF-8 decode, no `unwrap` on parse |
| Loopback video server exposure | another local process/browser connects | bind `127.0.0.1`, ephemeral port, 128-bit token, `Origin` check, session-scoped shutdown |
| Malicious APK | user installs malware | out of scope to prevent; we only transport. Do not auto-install anything |
| Update channel compromise | key/feed | offline key in CI secret, signed manifest, HTTPS, opt-in |
| Frontend XSS via device strings | package labels, log lines | Svelte escapes by default; **never** `{@html}` on device data (CI lint rule) |
| Privilege abuse by plugins | `fs`/`opener` scope bugs | minimal scope, no `fs` write outside captures, no arbitrary URL open |
| Secret leakage in logs | serials, host paths | log redaction: serial → last 4 chars, `%USERPROFILE%` → `~` |

## 7. Privacy audit (call-site rule)

CI grep asserts that these crates/APIs appear only in the files that legitimately own them:

```
reqwest / ureq / hyper client  → only src-tauri/src/updater_proxy.rs (and only when the user opted in)
fetch( in src/                 → only into the loopback video origin (checked by review + a lint pattern)
```

The only network destinations in the whole app must be: `127.0.0.1:<adb 5037>`, `127.0.0.1:<video port>`,
and (opt-in) the update endpoint. Anything else is a bug.

## 8. Logging and redaction

`tauri-plugin-log` to `%LOCALAPPDATA%/<app>/logs/`, rotating at 5 MB × 3. Levels: `error`/`warn` always;
`debug`/`trace` behind a setting (default off). Every adb invocation is logged as
`adb <subcommand> -s <serial4> <args-redacted>` — never full stdout unless trace is on, and never a full
serial.

## 9. Verification of this document

| Item | Status |
|---|---|
| Plugin versions | `VERIFIED` — `10-version-pin.md` |
| Capabilities semantics quote | `VERIFIED` — https://tauri.app/security/capabilities/ |
| `withGlobalTauri`, `security.csp`, `devtools` config keys | `VERIFIED` — https://schema.tauri.app/config/2 |
| CSP `http://127.0.0.1:*` is accepted by WebView2 in a Tauri page | `UNVERIFIED` — verify in M0 with the DevTools network panel |
| `updater.pubkey` / `endpoints` config shape | `UNVERIFIED` for 2.14.1 — read `https://v2.tauri.app/plugin/updater/` at M6 |