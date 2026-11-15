# Windows port — documents

Target: **Windows-only GUI**, **Tauri v2**, **proper installer**, **no MCP server**, `Sources/**` kept as
read-only reference. Every number is measured or labelled `ESTIMATE`. Every API claim carries a URL.

| Doc | Contents |
|---|---|
| [`00-target-and-constraints.md`](00-target-and-constraints.md) | Approved scope, non-goals, hard budgets, naming |
| [`04-feature-parity-plan.md`](04-feature-parity-plan.md) | Which upstream features ship in which milestone |
| [`05-repo-restructure.md`](05-repo-restructure.md) | Where new code lives; what is never moved |
| [`10-version-pin.md`](10-version-pin.md) | Every pinned version + source URL + verification date |
| [`11-tauri-architecture.md`](11-tauri-architecture.md) | Topology, workspace, modules, threading, Windows process rules |
| [`12-ipc-contract.md`](12-ipc-contract.md) | Command/event table, error codes, rate budgets, transport choice |
| [`13-mirroring-design.md`](13-mirroring-design.md) | Candidates A–D, protocol, latency budget, fallback ladder |
| [`14-frontend-design.md`](14-frontend-design.md) | Svelte 5 tree, state, lists, a11y, Windows feel |
| [`15-plugin-and-security.md`](15-plugin-and-security.md) | Plugin table, capabilities, CSP, threat model |
| [`16-build-package-release.md`](16-build-package-release.md) | Commands, `tauri.conf.json` skeleton, bundling, signing, CI |
| [`17-testing-strategy.md`](17-testing-strategy.md) | Traits, golden fixtures, fake-adb, E2E, perf tests |
| [`18-plan-and-backlog.md`](18-plan-and-backlog.md) | M0–M6, task table with acceptance criteria |
| [`19-risk-register.md`](19-risk-register.md) | Risk, likelihood, impact, trigger, mitigation, fallback |
| [`20-spikes.md`](20-spikes.md) | S1–S6: timebox, pass/fail metric, what changes if it fails |
| [`../adr/`](../adr/) | ADR-0001 framework, ADR-0002 mirroring, ADR-0003 repo strategy |
| [`../feature-matrix.md`](../feature-matrix.md) | Upstream feature inventory with tiers (written before this plan) |

Numbering follows the two planning prompts: `00–09` was the Gate 2 set, `10–20` the Gate 3 set. Files
`01/02/03/06/07/08` from the earlier numbering were folded into `ADR-0001`, `13-`, `11-`, `17-`, `16-`,
`19/20` so there is exactly one source of truth per topic.

## Status legend used in these docs

| Mark | Meaning |
|---|---|
| `VERIFIED` | Read from the pinned version's docs/source or measured on this machine, URL cited |
| `UNVERIFIED` | Believed but not executed; must be confirmed before it gates a decision |
| `ESTIMATE` | A guess, not a measurement |
| `SPIKE` | Cannot be known without building it; see `20-spikes.md` |
| `DECISION` | Locked by the user; changing it needs a new ADR |

## Open decision carried forward

The user's Gate 1 instruction was *"no live screen mirroring"*; the Gate 3 brief reverses that and makes
video the central risk. This plan therefore treats mirroring as **M3-optional and spike-gated** (S1–S3),
with `13-mirroring-design.md` written as *Proposed*. Nothing in M0–M2 depends on it. See
`ADR-0002`, Status: **Proposed — awaiting confirmation**.