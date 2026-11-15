# Contributing to this fork

Rules for human and AI contributors. Upstream's engineering rules still apply where the code is
unchanged — read `docs/upstream/CONTRIBUTING.md` (the original) first, then this file.

## Current phase

**Documentation phase.** `Sources/**` must not be modified. Allowed writes: `docs/**`, `NOTICE.md`,
`README.md`, `.gitignore`, `.gitattributes`, `.editorconfig`, and git metadata. Any PR that touches
`Sources/**` before the framework ADR is approved must be closed, not reviewed.

## Branch naming

| Prefix | Use |
|---|---|
| `feat/<topic>` | new capability |
| `fix/<topic>` | bug fix |
| `docs/<topic>` | documentation only |
| `chore/<topic>` | tooling, git, CI |
| `spike/<topic>` | throwaway experiment; never merged without an ADR |

## Commit convention

[Conventional Commits](https://www.conventionalcommits.org/). Scopes that match this repo:

```
docs(arch): add adb integration inventory
docs(agents): add root AGENTS.md
chore(git): add .gitattributes and fork NOTICE
docs(inventory): record coverage gaps
feat(win/adb): add AdbProcessRunner with CREATE_NO_WINDOW
```

One logical change per commit. Documentation that describes a behaviour change belongs in the **same
commit** as the change (the doc-drift rule).

## Doc-drift rule

If you change behaviour, update in the same commit:

1. the nearest `AGENTS.md` (once the tree exists),
2. the relevant `docs/architecture/*.md`,
3. `docs/feature-matrix.md` if a feature's tier or feasibility changed,
4. `NOTICE.md` if a source file changed for the first time.

## ADR rule

A new dependency, a new protocol, a new on-disk location, or a change of framework **requires an ADR**
in `docs/adr/` (template `ADR-0000-template.md`, status starts as `Proposed`). No dependency is added
to make a single task easier.

## ADB safety rules (mandatory for everything in this fork)

**Read-only allowlist** — these may run in automated tests and in scripts:

```
adb devices -l
adb -s <serial> get-state
adb -s <serial> shell getprop <name>
adb -s <serial> shell pm list packages
adb -s <serial> shell wm size | wm density
adb -s <serial> exec-out screencap -p
adb -s <serial> shell dumpsys ...        (read-only dumps only)
adb -s <serial> logcat -d
```

**Explicit human approval required** for anything that mutates: `install`, `uninstall`,
`push`/`pull` (writes to host or device), `rm`, `pm clear`, `pm reset-permissions`, `settings put`,
`settings delete`, `setprop`, `am force-stop`, `svc wifi|data`, `cmd connectivity`, `wm density <n>`,
`emu …`, `reboot`, `root`, `remount`, `kill-server`, `start-server`.

Additional rules:

1. **Always** target with `-s <serial>`. Never run a mutating command with an implicit device.
2. **Never** assume a single device; enumerate first.
3. **Never** run `adb` in a unit test. Integration tests are behind an explicit flag and a serial env var.
4. **Never** print full device serials in logs. Use a stable hash or the last 4 characters.
5. A command string that reaches `adb shell` must be quoted with the project's `shellQuoted` equivalent
   (single-quote + `'\''` escaping, `Sources/Core/Support.swift:14`). No exceptions, including "it's just
   a number".
6. Do not add features that need `root`.

## Data safety and privacy

- No device data leaves the machine. No telemetry, no crash upload, no analytics — this is a stated
  product property (`SECURITY.md:9-16`), not a preference.
- App-data features may only read `debuggable=true` builds, via `run-as`, and must never write to the
  device.
- Redact serials and absolute host paths in anything that could be shared (issues, logs you attach).
- Sanitize any device-supplied filename before using it as a local path (see
  `docs/architecture/12-risks-and-tech-debt.md` R12 for a known upstream example).

## PR checklist

- [ ] `git diff --stat upstream-baseline-9f37391 -- Sources` is empty **or** the PR explicitly implements
      an approved ADR.
- [ ] Docs updated in the same commit (doc-drift rule).
- [ ] No new dependency without an ADR.
- [ ] No new network call. If one is unavoidable, it is documented in `docs/architecture/09-cross-cutting.md`
      and is opt-out.
- [ ] No `adb` command that mutates without the safety section above being satisfied.
- [ ] Commands in docs are either verified and marked so, or marked `UNVERIFIED`.
- [ ] Every non-obvious claim has a `path:line` citation.
- [ ] `LICENSE` unchanged; `NOTICE.md` updated if a source file changed.
- [ ] Commit messages are Conventional Commits; no squashing of upstream history; no force-push.

## Reviewing AI-generated PRs

Require evidence, not confidence: a citation for every behavioural claim, `UNVERIFIED` for anything not
executed, and an explicit list of files the author did **not** read. The repo's `docs/coverage.md` is the
model to follow.