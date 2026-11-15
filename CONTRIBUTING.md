# Contributing

Thanks for looking. This is a **Windows-only fork** of [Oh My Android](https://github.com/ateymoori/oh-my-android):
the shipped product is a Tauri v2 / Rust / Svelte 5 desktop app, and the upstream Swift sources under
[`Sources/`](Sources/) are a **read-only reference implementation**.

Before you write code, read [`AGENTS.md`](AGENTS.md) — it lists the invariants this repository enforces,
some of them in CI.

- [Ways to contribute](#ways-to-contribute)
- [Ground rules](#ground-rules)
- [Development setup](#development-setup)
- [Gates you must pass](#gates-you-must-pass)
- [Branch and commit conventions](#branch-and-commit-conventions)
- [Pull requests](#pull-requests)
- [Changing behaviour: docs and ADRs](#changing-behaviour-docs-and-adrs)
- [ADB safety rules](#adb-safety-rules)
- [Releases](#releases)
- [Reporting bugs and security issues](#reporting-bugs-and-security-issues)

## Ways to contribute

- **Bug reports** — use the issue template. Include Windows version, WebView2 version, `adb version`, and the
  exact error text the app showed.
- **Device reports** — which phones/emulators/`adb` versions the list handles correctly, and which output
  breaks the parser. Real `adb devices -l` lines are gold: they become fixtures in
  `crates/adbkit/tests/`.
- **Docs** — the parity plan and measurement log are always behind reality.
- **Code** — start with [`docs/windows-port/18-plan-and-backlog.md`](docs/windows-port/18-plan-and-backlog.md).

Not wanted: the MCP server, macOS/Linux support, cross-platform abstraction layers, telemetry, or any
feature that installs software on the user's machine for them.

## Ground rules

These come from `AGENTS.md` and are not negotiable in review:

1. **`Sources/**` is read-only.** It is the behavioural specification. A PR that edits it is closed, not
   reviewed. (A deliberate exception requires an ADR and maintainer sign-off.)
2. **Only `crates/adbkit` may spawn a process.** CI greps for `Command::new` elsewhere and fails the build.
   argv arrays, never shell strings.
3. **`adb kill-server` is destructive.** It disconnects every adb client on the machine. It stays behind a
   confirmation dialog, is never invoked automatically, and never in a test.
4. **Offline-first.** No runtime downloads, no CDN, no telemetry. CI rejects remote asset URLs in the UI
   source.
5. **Monochrome UI.** State is carried by icon *shape*, text and weight — never hue.
6. **Windows-only.** No `#[cfg(target_os = "linux")]` branches "for portability".
7. **Tauri v2 APIs only.** `tauri::api::*` and `allowlist` are v1 and are rejected by CI.

## Development setup

Requirements: **Windows 10/11 x64**, Rust **1.90** (`rustup`, pinned in `rust-toolchain.toml`), Node **24**
(`.nvmrc`), the MSVC C++ build tools, WebView2 (preinstalled on Windows 11), and an Android SDK providing
`platform-tools\adb.exe`.

```powershell
git clone https://github.com/bramburn/oh-my-android.git
cd oh-my-android
npm ci                 # exact lockfile install
npm run tauri dev      # dev window with hot reload
```

Full build/install/usage instructions: [`README.md`](README.md).

## Gates you must pass

CI runs exactly this on Windows, and a PR cannot merge until it is green:

```powershell
npm run check                      # svelte-check (also part of npm run build)
npm run build                      # svelte-check + vite; required before cargo, see note
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace             # 46 tests, no device required
npm run tauri build                # produces the NSIS installer
```

Plus three CI guards: no `Command::new` outside `crates/adbkit`, no Tauri v1 APIs, no remote asset URLs.

> **Why frontend first:** `tauri::generate_context!` embeds `dist/` at compile time, so `cargo clippy` /
> `cargo test` fail on a clean checkout until `npm run build` has run.

New logic needs a test. If it touches adb parsing, add a fixture under `crates/adbkit/tests/fixtures/` and a
case in the golden test — not a mock of your own parser.

## Branch and commit conventions

| Prefix | Use |
|---|---|
| `feat/<topic>` | new capability |
| `fix/<topic>` | bug fix |
| `docs/<topic>` | documentation only |
| `chore/<topic>` | tooling, git, CI |
| `spike/<topic>` | throwaway experiment; never merged without an ADR |

`main` is protected: **no direct pushes except by the maintainer**, and every other change arrives as a PR
from your own fork. Keep branches short-lived; one topic per branch.

Commits follow [Conventional Commits](https://www.conventionalcommits.org/) with the scopes used here:

```
feat(win): add setup guidance screen
fix(adbkit): tolerate a missing model: field
docs(port): record installer measurements
chore(ci): require the windows check on main
```

One logical change per commit. Behaviour and the documentation that describes it belong in the **same**
commit — a doc-drift check in review is simply "does the README/log still describe what this code does?".

## Pull requests

1. Fork, branch off `main`, push to your fork.
2. Open the PR against `main`. Fill in the template; the checklist mirrors the CI gates.
3. Keep the PR focused. Unrelated cleanups go in a separate PR — reviewers are volunteers.
4. Screenshots for UI changes. A green `svelte-check` says nothing about rendering; two of the bugs in the
   MVP log were invisible to type-checking and obvious in a screenshot.
5. Respond to review by pushing commits (no force-push during review, please). Squash on merge is the
   default, so history tidiness is handled for you.

A maintainer reviews every PR. Expect questions about: does this spawn anything, does it survive offline,
does it handle a device that reports `offline`/`unauthorized`, and what happens on the error path.

## Changing behaviour: docs and ADRs

- **New dependency or plugin** → an ADR in [`docs/adr/`](docs/adr/) using `ADR-0000-template.md`, with the
  rejected alternatives and their measured cost.
- **New IPC command** → add a row to [`docs/windows-port/12-ipc-contract.md`](docs/windows-port/12-ipc-contract.md),
  a stable error code in `src-tauri/src/error.rs`, and a typed wrapper in `src/lib/ipc.ts`.
- **Deviation from the brief** → a numbered `D`-row in
  [`docs/windows-port/21-mvp-log.md`](docs/windows-port/21-mvp-log.md) §3, with the reason and the measurement.
- **An override of an `AGENTS.md` invariant** → an `O`-row in §2 of the same log. Overrides are retired
  deliberately, never drifted away from.

## ADB safety rules

- Every adb interaction goes through [`crates/adbkit`](crates/adbkit/): `runner.rs` (spawn with
  `CREATE_NO_WINDOW`, timeout, job object) and a typed parser — never string formatting into a shell.
- Treat device-supplied strings (model, serial, package names) as hostile: no path joins, no shell, no
  file names.
- Mutating operations need a confirmation in the UI and a note in the docs. `kill-server` is the current
  example.
- Never run adb against a real device from a test. Use fixtures and the future `crates/fake-adb`.

## Releases

Maintainer only. Bump the version in `src-tauri/tauri.conf.json` **and** `package.json`, commit, then push a
tag:

```powershell
git tag v0.1.1
git push origin v0.1.1
```

`.github/workflows/release.yml` refuses to build when the tag and the manifests disagree, runs the full gate
set, builds the NSIS installer, writes `SHA256SUMS.txt`, and publishes the GitHub release. Details:
[`docs/windows-port/16-build-package-release.md`](docs/windows-port/16-build-package-release.md).

## Reporting bugs and security issues

- Bugs: open an issue with the template. Paste the exact error text and the setup screen's report (the wrench
  button) — it says which SDK folder and `adb` version the app used.
- Security: **do not open a public issue.** See [`SECURITY.md`](SECURITY.md).

## Licence

Contributions are accepted under the MIT licence in [`LICENSE`](LICENSE). By opening a PR you agree your
contribution may be distributed under it. Third-party assets need their licence and attribution recorded in
[`NOTICE.md`](NOTICE.md) in the same PR — see [ADR-0009](docs/adr/ADR-0009-icons-webfont.md) for the pattern.