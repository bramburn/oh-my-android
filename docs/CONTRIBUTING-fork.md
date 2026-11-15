# Fork conventions (pointer)

The contributor guide now lives at [`../CONTRIBUTING.md`](../CONTRIBUTING.md) — GitHub renders it on the
"Contributing" tab, and it is the single source of truth for setup, gates, branch/commit conventions, PR
expectations and the ADB safety rules.

Two things specific to working *in this repository* (as opposed to understanding it):

- **Invariants** are in [`../AGENTS.md`](../AGENTS.md). Some of them are enforced by CI guards rather than
  by review (`Command::new` outside `crates/adbkit`, Tauri v1 APIs, remote asset URLs).
- **`Sources/**` is read-only.** It is the behavioural specification taken verbatim from upstream. Do not
  edit, reformat or "fix" it; document the intended behaviour in `docs/` instead.

Deep background, if you are changing architecture rather than fixing a bug:

| Document | Why you would open it |
|---|---|
| [`windows-port/11-tauri-architecture.md`](windows-port/11-tauri-architecture.md) | module boundaries, threading, Windows process rules |
| [`windows-port/12-ipc-contract.md`](windows-port/12-ipc-contract.md) | commands, events, error taxonomy |
| [`windows-port/17-testing-strategy.md`](windows-port/17-testing-strategy.md) | what to test and at which layer |
| [`windows-port/21-mvp-log.md`](windows-port/21-mvp-log.md) | overrides and deviations currently in force |
| [`adr/`](adr/) | why the stack looks like this |