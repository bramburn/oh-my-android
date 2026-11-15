# Upstream docs — preserved verbatim

These files are byte-for-byte copies of the upstream repository at the fork point
`9f37391eef056c0e6a45327b56670f200bb62460`, kept so the original documentation survives the fork's new
root `README.md`.

| File | Original path |
|---|---|
| `README.md` | `README.md` |
| `CONTRIBUTING.md` | `CONTRIBUTING.md` |
| `SECURITY.md` | `SECURITY.md` |
| `COMPARISON.md` | `docs/COMPARISON.md` |

**Do not edit these.** Behaviour that changes in this fork is documented in `docs/architecture/`, not
here.

Known link breaks, caused only by the relocation (not by the upstream content):

- `README.md` links `docs/COMPARISON.md` → resolves to `docs/upstream/docs/COMPARISON.md` (actual copy:
  `docs/upstream/COMPARISON.md`).
- `README.md` links `LICENSE` → the licence is at the repository root, three levels up.

Images referenced by the upstream README (`docs/demo.gif`, `docs/panel.jpg`, …) resolve correctly only
when that README is viewed from the repository root; they live in `docs/` at the root.