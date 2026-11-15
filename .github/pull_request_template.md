## What and why

<!-- One paragraph. Link the issue this closes, if any. -->

Closes #

## How

<!-- What changed, and why this way. Note anything a reviewer should not "fix". -->

## Checklist

- [ ] `npm run build` passes (svelte-check + vite), and `cargo fmt --all --check`,
      `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` are clean
- [ ] No `Command::new` outside `crates/adbkit`; adb is still invoked as an argv array, never a shell string
- [ ] New logic has a test; new adb output has a fixture under `crates/adbkit/tests/fixtures/`
- [ ] UI changes: screenshot attached, and it is still **monochrome** (state by shape + text, never hue)
- [ ] Offline-first preserved: no new network call, no CDN, no remote asset
- [ ] Docs updated in this PR: README / `docs/windows-port/12-ipc-contract.md` for IPC /
      `21-mvp-log.md` for deviations and measurements
- [ ] If this mutates a device or the adb server: the UI confirms first, and the docs say so
- [ ] If this adds a dependency, plugin or asset: an ADR exists and `NOTICE.md` is updated
- [ ] `Sources/**` untouched (it is the read-only reference implementation)