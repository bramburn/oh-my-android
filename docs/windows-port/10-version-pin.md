# 10 — Version pin

Retrieved **2026-11-15** from the registries (not recalled). Re-verify before M0 starts; update the
`verified-on` column, never delete rows.

Method: `https://crates.io/api/v1/crates/<name>` → `crate.max_stable_version`;
`https://registry.npmjs.org/<pkg>/latest` → `version`; `https://api.github.com/repos/<o>/<r>/releases/latest`.

## 1. Pinned versions

| Component | Pinned | Source | Verified | Notes |
|---|---|---|---|---|
| Rust (rustc/cargo) | **1.89.0** | local `rustc --version` → `1.89.0 (29483883e 2025-08-04)` | 2026-11-15 | Pin via `rust-toolchain.toml`; target `x86_64-pc-windows-msvc` (`UNVERIFIED` that the MSVC target is installed here) |
| MSVC build tools | VS 2022 Build Tools, MSVC 14.4x | — | `UNVERIFIED` | Required by `*-pc-windows-msvc` |
| Node.js | **24.11.0** (Node 24 LTS line) | local `node --version` | 2026-11-15 | Vite 8 requires ≥20.19/22.12; 24 satisfies it |
| npm | 11.21.1 | local | 2026-11-15 | |
| `tauri` (crate) | **2.12.5** | crates.io | 2026-11-15 | |
| `tauri-build` | **2.7.4** | crates.io | 2026-11-15 | build-dependency |
| `wry` | 0.57.2 | crates.io | 2026-11-15 | transitively pinned by `tauri`; do not pin directly |
| `tao` | 0.37.2 | crates.io | 2026-11-15 | same |
| `@tauri-apps/cli` | **2.12.4** | npm | 2026-11-15 | |
| `@tauri-apps/api` | **2.12.4** | npm | 2026-11-15 | must match the crate's minor |
| `tokio` | 1.54.1 | crates.io | 2026-11-15 | `features = ["rt-multi-thread","process","net","sync","time","macros","io-util"]` |
| `serde` / `serde_json` | 1.0.229 / current | crates.io | 2026-11-15 | |
| `thiserror` | 2.0.21 | crates.io | 2026-11-15 | |
| `regex` | 1.13.1 | crates.io | 2026-11-15 | prefer hand parsers where the format is stable |
| `rusqlite` | 0.43.0 | crates.io | 2026-11-15 | `bundled` feature → no external SQLite; +`ESTIMATE` 1 MB |
| `axum` | 0.8.9 | crates.io | 2026-11-15 | only if mirroring M3 lands (loopback video server) |
| `tauri-plugin-log` | 2.10.2 | crates.io | 2026-11-15 | |
| `tauri-plugin-store` | 2.5.1 | crates.io | 2026-11-15 | settings persistence |
| `tauri-plugin-dialog` | 2.8.4 | crates.io | 2026-11-15 | native file pickers |
| `tauri-plugin-fs` | 2.7.1 | crates.io | 2026-11-15 | scoped; see `15-plugin-and-security.md` |
| `tauri-plugin-single-instance` | 2.5.5 | crates.io | 2026-11-15 | |
| `tauri-plugin-window-state` | 2.5.0 | crates.io | 2026-11-15 | panel position memory |
| `tauri-plugin-clipboard-manager` | 2.5.0 | crates.io | 2026-11-15 | screenshot → clipboard (upstream parity) |
| `tauri-plugin-opener` | 2.7.3 | crates.io | 2026-11-15 | open captures folder |
| `tauri-plugin-os` | 2.4.0 | crates.io | 2026-11-15 | platform hints |
| `tauri-plugin-notification` | 2.8.1 | crates.io | 2026-11-15 | install finished / device lost |
| `tauri-plugin-updater` | 2.14.1 | crates.io | 2026-11-15 | opt-in only; new keypair |
| `svelte` | **5.57.3** | npm | 2026-11-15 | runes API |
| `vite` | **8.3.8** | npm | 2026-11-15 | |
| `tailwindcss` | **4.3.5** | npm | 2026-11-15 | v4 CSS-first config |
| `typescript` | **7.0.2** | npm | 2026-11-15 | ⚠ see §3 |
| `vitest` | 5.1.0 | npm | 2026-11-15 | |
| `scrcpy` (server jar + exe) | **v5.0.1** (2026-10-08) | GitHub releases API | 2026-11-15 | only if ADR-0002 resolves to A/B/C |
| WebView2 Evergreen | minimum TBD | — | `UNVERIFIED` | set `bundle.windows.minimumWebview2Version` after S5 |
| `platform-tools` (adb) | **not bundled by default** | — | `UNVERIFIED` | licensing (`docs/legal.md` §2); if bundled, pin the SDK revision string |

## 2. Version-pinning mechanics

| Artifact | Content |
|---|---|
| `rust-toolchain.toml` | `[toolchain] channel = "1.89.0"`, `components = ["clippy","rustfmt"]`, `targets = ["x86_64-pc-windows-msvc"]` |
| `src-tauri/Cargo.toml` | exact `=` pins for `tauri`, `tauri-build`; caret for the rest; `Cargo.lock` **committed** |
| `package.json` | exact versions (no `^`), `package-lock.json` committed |
| `.nvmrc` / `package.json > engines` | `24.11.0` / `>=24.11.0` |
| `src-tauri/tauri.conf.json` | `bundle.windows.minimumWebview2Version` set once S5 reports |
| `docs/windows-port/10-version-pin.md` | this table — the human-readable source of truth |

Renovate/Dependabot: weekly, grouped, **never** auto-merge a `tauri`/`wry`/`tao` bump (they change IPC and
browser behaviour).

## 3. Version risks to resolve in M0

| Risk | Detail | Resolution |
|---|---|---|
| **TypeScript 7.0.2** | TS 7 is the native (Go) compiler; plugin/`svelte-check` support may lag | If `svelte-check`/`vite-plugin-svelte` misbehave, pin `typescript@5.9.x` and record the downgrade here. `UNVERIFIED` |
| **Vite 8 + `@sveltejs/vite-plugin-svelte`** | major mismatch is the classic failure | `npm create tauri-app` template decides; record the resolved pair |
| **Tailwind 4** | CSS-first `@theme` config, no `tailwind.config.js` | follow v4 docs; do not copy v3 snippets |
| **Rust 1.89 vs `tauri` 2.12 MSRV** | if `tauri` 2.12 needs a newer rustc, cargo errors immediately | bump `rust-toolchain.toml` and this table; never patch vendored code |
| **`rusqlite 0.43` + `bundled`** | compiles SQLite with `cc` → needs MSVC | verify at M0 with `cargo build` |

## 4. API claims made elsewhere in this plan

| Claim | Where used | Status |
|---|---|---|
| `tauri::ipc::Channel<T>` streams many messages after a command returns; ordering guaranteed; `Channel<Vec<u8>>` is sent as **JSON**, while `Channel<InvokeResponseBody>`/`Response` delivers an `ArrayBuffer` | `12-ipc-contract.md` §4 | **VERIFIED** — https://docs.rs/tauri/2.12.5/tauri/ipc/struct.Channel.html |
| `register_asynchronous_uri_scheme_protocol` hands the responder an `http::Response<Vec<u8>>` → body is fully materialized (buffered), not a stream | why video uses loopback HTTP | **VERIFIED** — signature at https://docs.rs/tauri/2.12.5/tauri/struct.Builder.html |
| Custom-protocol pages get a different `Origin` on Windows | CSP/`connect-src` design | **VERIFIED** (same page: "Warning: Pages loaded from a custom protocol will have a different Origin on …") |
| `externalBin` requires `binary-name{-target-triple}{.exe}` suffix | why `platform-tools` uses `bundle.resources` | **VERIFIED** — https://schema.tauri.app/config/2 (`BundleConfig.externalBin`) |
| `bundle.resources` accepts a list or a `{source: target}` map; files land under `$RESOURCES` | `platform-tools` bundling | **VERIFIED** — same schema (`BundleResources`) |
| `WebviewInstallMode` = `skip \| downloadBootstrapper \| embedBootstrapper \| offlineInstaller \| fixedRuntime` | installer strategy | **VERIFIED** — same schema |
| `WebviewWindowBuilder::additional_browser_args` is Windows-only | panel/no-activate args | **VERIFIED** — https://docs.rs/tauri/2.12.5/tauri/webview/struct.WebviewWindowBuilder.html (marked "Platform-specific macOS/Linux/Android/iOS: Unsupported"; also warns different args need different data directories) |
| Capabilities are per-window JSON files auto-enabled unless `app.security.capabilities` lists them explicitly | least privilege | **VERIFIED** — https://tauri.app/security/capabilities/ |
| **WebCodecs `VideoDecoder` availability in WebView2** | the entire video decision | **UNVERIFIED — SPIKE S3.** Microsoft's WebView2 surface docs do not state it; no local test possible without a build |
| `avc: { format: 'annexb' }`, `optimizeForLatency`, `hardwareAcceleration` support in WebView2's decoder | S3 | **SPIKE S3** (spec-level support ≠ WebView2 implementation) |
| `tauri-driver` + msedgedriver E2E on Windows | `17-testing-strategy.md` §5 | **UNVERIFIED** |

## 5. Rule

A version bump is a PR that touches exactly: the lockfile, the relevant `Cargo.toml`/`package.json`, and
this table. Bumping `tauri`/`wry`/`tao` additionally requires re-running spikes S1 and S3.