# AGENTS.md — docs/windows-port (scope: docs/windows-port/)

Read the root `AGENTS.md` first. This directory is the **Windows plan**; it contains no code.

## Rules for this directory

1. **No implementation claims without a status mark**: `VERIFIED (source)`, `UNVERIFIED`, `ESTIMATE`,
   `SPIKE`, `DECISION`. See `README.md` §Status legend.
2. **API claims need a URL.** Versions and API behaviour come from the pinned version's docs or
   source — never from memory. Tauri v1 APIs (`tauri::api::*`, `allowlist`, `tauri.conf.json > tauri`) are
   bugs: flag them.
3. **Numbers are measured or labelled.** A latency/size/CPU figure without one of those marks is a review
   reject.
4. **One source of truth per topic.** Do not re-describe architecture here; link to `11-tauri-architecture.md`.
   If a doc and code disagree, the doc is wrong until proven otherwise — then fix the doc in the same commit.
5. **Upstream behaviour claims cite `Sources/**` as `path:line`**, same as `docs/architecture/`.
6. **Decisions are ADRs.** A new dependency, protocol, on-disk location, or framework change goes to
   `../adr/` as `Proposed` first (`../adr/ADR-0000-template.md`).
7. **Never edit** `docs/upstream/**`, `Sources/**`, or `.github/workflows/build.yml` from a task started here.

## File map

| File | Owns |
|---|---|
| `00-target-and-constraints.md` | scope decisions, non-goals, budgets, naming |
| `04-feature-parity-plan.md` | which upstream feature ships in which milestone |
| `05-repo-restructure.md` | directory layout, what is never touched |
| `10-version-pin.md` | every pinned version + API-claim status table |
| `11-tauri-architecture.md` | topology, workspace, modules, threading, Windows process rules |
| `12-ipc-contract.md` | commands, events, error codes, transport, rate budgets |
| `13-mirroring-design.md` | video candidates A–D, wire protocol, latency budget |
| `14-frontend-design.md` | Svelte structure, state, lists, shortcuts, assets |
| `15-plugin-and-security.md` | plugins, capabilities, CSP, threat model |
| `16-build-package-release.md` | commands, `tauri.conf.json`, bundling, signing, CI |
| `17-testing-strategy.md` | fixtures, fake-adb, E2E, perf gates |
| `18-plan-and-backlog.md` | M0–M6 tasks with acceptance criteria |
| `19-risk-register.md` | scored risks + top 5 |
| `20-spikes.md` | S1–S6: timebox, metric, plan change |

## Before you write code from this plan

- M3 (mirroring) is **blocked** until ADR-0002 is Accepted and S1–S3 have numbers.
- Anything marked `UNVERIFIED` in `10-version-pin.md` §4 must be confirmed in the milestone that first
  depends on it, and the table updated (never deleted).
- Update this directory when a milestone changes a decision; the doc-drift rule applies here hardest,
  because the code does not exist yet and these files *are* the product definition.