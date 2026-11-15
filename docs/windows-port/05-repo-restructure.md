# 05 — Repo restructure

`DECISION` recorded in `../adr/ADR-0003-repo-strategy.md`: **strangler-fig by addition**, not a new
repository and not a `reference/` move.

## 1. Target tree

```
.
├── AGENTS.md                     # updated for the Windows app (root)
├── README.md  NOTICE.md  LICENSE  .gitattributes  .editorconfig  .gitignore
├── Cargo.toml                    # NEW: workspace root
├── rust-toolchain.toml           # NEW
├── package.json  package-lock.json  .nvmrc  vite.config.ts  tsconfig.json  index.html   # NEW
├── crates/
│   ├── adbkit/                   # NEW: all adb access; no tauri deps
│   ├── mirror/                   # NEW: scrcpy client (feature-gated)
│   └── fake-adb/                 # NEW: test double binary
├── src/                          # NEW: Svelte 5 + TS UI (replaces nothing)
├── src-tauri/                    # NEW: Tauri shell, commands, capabilities, resources
├── tests/fixtures/               # NEW: UI-level fixtures
├── spikes/                       # NEW: throwaway, not in the workspace (excluded in Cargo.toml)
├── docs/                         # existing + windows-port/ + adr/
│   └── upstream/                 # verbatim upstream docs (do not edit)
├── Sources/                      # UNCHANGED — reference implementation
├── project.yml  OhMyAndroid.xcodeproj (generated)  Scripts/  mcpb/  skills/  .github/workflows/build.yml
└── .github/workflows/windows.yml # NEW, sits beside build.yml
```

## 2. What is never touched

| Path | Rule |
|---|---|
| `Sources/**` | read-only reference (D5). No edits, moves, deletions, or "fixes" |
| `project.yml`, `Scripts/`, `mcpb/`, `skills/`, `.claude-plugin/` | unchanged |
| `.github/workflows/build.yml` | unchanged (D4); macOS CI must keep passing |
| `LICENSE` | byte-for-byte; verified in CI-ish review (`docs/git-setup.md` §7) |
| `docs/upstream/**` | verbatim copies |

## 3. Relationship to `Sources/` over time

| Milestone | `Sources/` role |
|---|---|
| M0–M5 | primary reference; every Rust module names its Swift counterpart |
| M6 (1.0) | archive candidate. Keep the directory, add `Sources/README-REFERENCE.md` explaining it is not built by the Windows pipeline and why it is retained (lineage + MIT attribution + behaviour questions) |
| post-1.0 | optionally stop running macOS CI, never delete the directory without a decision |

## 4. Why not a fresh repository

| Option | Verdict |
|---|---|
| **Add new dirs in this fork** (chosen) | keeps upstream remote, tags, history and MIT attribution in one place; `git blame` on the reference code stays; one CI to reason about; fork can still `merge upstream` to track fixes |
| New repo, this fork archived | loses the reference's proximity, splits issue history, doubles maintenance for two more months |
| Rewrite on a branch with no docs | unreviewable; the docs are the only reason a port this size is estimable |
| Move `Sources/` to `reference/` | churn in 98 files with zero benefit, and it breaks upstream merge (D5 forbids it) |

## 5. Cleanup rules (transient files)

| Path | Rule |
|---|---|
| `spikes/` | excluded from the workspace (`exclude = ["spikes"]`); each spike has a README with measurement + verdict; delete a spike after its ADR is written, keep its numbers in `20-spikes.md` |
| `src-tauri/target`, `node_modules`, `dist` | git-ignored (already covered by `.gitignore`) |
| `.scratch/` | local only, git-ignored |
| fixtures | keep, they are tests; redact serials |

## 6. Ownership map (who edits what)

| Concern | Path | Reviewer focus |
|---|---|---|
| adb semantics, parsing | `crates/adbkit/**` | fixtures, quoting, error mapping |
| Tauri wiring, IPC | `src-tauri/**` | thin commands, capability scope |
| UI | `src/**` | no direct `invoke`, a11y, virtualized lists |
| Video | `crates/mirror/**`, `src-tauri/src/media/**` | protocol pin, frame `close()`, token auth |
| Packaging | `src-tauri/tauri.conf.json`, `.github/workflows/windows.yml` | size, signing, WebView2 mode |
| Docs | `docs/**`, `AGENTS.md` | citations, `UNVERIFIED` marks |

## 7. Order of creation (M0)

1. `Cargo.toml` workspace + `rust-toolchain.toml` + `.nvmrc`
2. `crates/adbkit` (compiles, has traits + one parser + fixtures)
3. `src-tauri` minimal shell + `capabilities/` + `tauri.conf.json`
4. `src/` minimal Svelte app with the panel skeleton
5. `.github/workflows/windows.yml`
6. `AGENTS.md` for each new directory
7. `spikes/` scaffolding

Nothing in step 1–7 modifies `Sources/**`.