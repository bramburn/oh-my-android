# NOTICE

This repository is a **modified fork**.

| Item | Value |
|---|---|
| Upstream project | Oh My Android |
| Upstream URL | https://github.com/ateymoori/oh-my-android |
| Upstream author | Amir Teymoori / Royan AB |
| Upstream licence | MIT (see `LICENSE`, kept byte-for-byte) |
| Fork point (commit) | `9f37391eef056c0e6a45327b56670f200bb62460` ("docs: Android-first README intro") |
| Fork baseline tag | `upstream-baseline-9f37391` |
| Upstream release at fork point | `v0.4.6` |
| Fork URL | https://github.com/bramburn/oh-my-android |
| Fork maintainer | bramburn |
| Fork purpose | Document and then, if approved, re-implement this product as a Windows-first GUI |

## What this fork changes

At the fork point, the changes are **documentation and repository hygiene only**:

- `docs/` gains an engineering documentation set (inventory, coverage, assumptions, open questions,
  architecture deep dive, feature matrix) and the original upstream docs under `docs/upstream/`.
- `README.md` at the root is replaced by a fork README; the upstream README is preserved verbatim at
  `docs/upstream/README.md`.
- `.gitattributes`, `.editorconfig`, and `.gitignore` are added/extended; remotes, branches and tags are
  reorganised (`origin` = this fork, `upstream` = the original).
- `NOTICE.md`, `docs/legal.md`, `docs/git-setup.md`, `docs/CONTRIBUTING-fork.md` are added.

**No application source file is modified in the fork's documentation phase.** Any future change to
`Sources/**` will be recorded here and in the commit history.

## Alternative dates and copyright

The upstream copyright notice is retained unchanged in `LICENSE`. The MIT licence requires the
copyright notice and permission notice to be included in all copies or substantial portions of the
software; keeping `LICENSE` intact satisfies that.

The name "Oh My Android", the bundle identifier `se.royan.ohmyandroid`, the app icon, the Homebrew tap,
and the Sparkle feed URL/EdDSA key in `project.yml` are the upstream author's identity and **must not be
reused** by a fork that publishes to end users. See `docs/legal.md` for the full obligation list.

## Third-party assets in the Windows application

The Windows app (`src-tauri/`, `src/`, `crates/`) bundles the following third-party assets. These
obligations apply to every distributed build (including the installer) and must not be removed.

| Asset | Version (pinned) | Licence | Obligation |
|---|---|---|---|
| Font Awesome Free — **Solid icons** (`@fortawesome/free-solid-svg-icons`) | 7.3.1 | **CC BY 4.0** (icons) / MIT (code) | Attribution required — see below |
| Font Awesome Free — code (`@fortawesome/fontawesome-svg-core`) | *not bundled* (removed, see ADR-0008) | MIT | n/a |
| Tauri / `wry` / `tao` / `windows-sys` | see `Cargo.lock` | MIT or Apache-2.0 | reproduce notices in the installer |
| Svelte, Vite, TypeScript | see `package-lock.json` | MIT | reproduce notices in the installer |

### Font Awesome Free — required attribution

> Font Awesome Free by Font Awesome — https://fontawesome.com
> Icons: CC BY 4.0 — https://creativecommons.org/licenses/by/4.0/
> Code: MIT — https://opensource.org/licenses/MIT

The glyphs used are: `arrows-rotate`, `play`, `stop`, `mobile-screen` (Solid). They are shipped as inline
SVG path data in the frontend bundle; the licence does **not** permit removing this attribution when the app
is redistributed. Changed files, if any, are limited to the four icon definitions being consumed unmodified.
