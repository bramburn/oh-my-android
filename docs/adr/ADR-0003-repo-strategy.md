# ADR-0003 — Repository strategy for the Windows app

- Status: **Accepted**
- Date: 2026-11-15
- Deciders: bramburn
- Supersedes: none

## Context

The fork must host a Windows Tauri app while keeping the upstream Swift app as the reference
implementation, and must keep macOS CI green (`00-target-and-constraints.md` D4, D5).

## Decision

Add the Windows app as **new top-level directories in this repository** (`crates/`, `src/`, `src-tauri/`,
`Cargo.toml`, frontend config), leaving `Sources/**`, `project.yml`, `Scripts/**`, `mcpb/**`,
`skills/**` and `.github/workflows/build.yml` untouched. Stub macOS CI unchanged; add
`.github/workflows/windows.yml` beside it.

## Consequences

- Positive: one history, one attribution chain, upstream merges still possible, reference code adjacent to
  the port, `git blame` intact, no issue/PR split.
- Negative: the repo is bilingual (Swift + Rust/TS) and grows two toolchains; contributors must read
  `AGENTS.md` to know which half they are in. Mitigated by a root `AGENTS.md` section and per-directory
  `AGENTS.md` files.
- Neutral: `Sources/` becomes archival after 1.0 but is never deleted without a decision.

## Alternatives considered

| Option | Why not |
|---|---|
| Fresh repo | loses proximity, attribution chain and upstream tracking; more work |
| Move `Sources/` to `reference/` | 98-file churn for no behavioural gain; breaks upstream merges |
| Branch-only rewrite with no docs | unreviewable |

## Evidence

`05-repo-restructure.md`, `docs/git-setup.md` §4.