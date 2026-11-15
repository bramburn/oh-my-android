# ADR-0009 — Font Awesome as a bundled icon **font** (supersedes ADR-0008)

- **Status:** Accepted (2026-11-15)
- **Supersedes:** ADR-0008 (icon *definitions* rendered by a local component)
- **Related:** ADR-0007 Amendment 2 §4, `NOTICE.md` (attribution), `docs/windows-port/14-frontend-design.md`

## Context

ADR-0008 chose `@fortawesome/free-solid-svg-icons` plus a 20-line local renderer, because it was the
smallest possible footprint: **+2.1 kB gzip** for four glyphs, no font files, no core runtime.

The setup-guidance milestone changes the requirement in two ways:

1. **Icon count grows from 4 to 13** across three semantic groups (status, device, action). Each glyph is
   ~0.5 kB of path data, so the definitions approach grows linearly and now needs an icon *map* config.
2. **The brief explicitly requires the font package bundled offline** (`@fortawesome/fontawesome-free`
   via npm, Vite-emitted woff2, no CDN, CSP `font-src 'self'`), with the Font Awesome licence obligations
   recorded in `NOTICE.md`.

## Decision

Adopt `@fortawesome/fontawesome-free@7.3.1` as a **locally bundled icon font**:

- `import '@fortawesome/fontawesome-free/css/fontawesome.min.css'` + `.../css/solid.min.css` in
  `src/main.ts` — **not** `all.min.css`, which also pulls the Regular and Brands faces (~2 extra fonts and
  ~230 kB for glyphs this app never renders).
- Vite copies the referenced `fa-solid-900.woff2` into `dist/assets/` at build time, so the app has **no
  runtime network dependency**. CSP: `font-src 'self'`.
- `src/lib/FaIcon.svelte` is a thin wrapper over `<i class="fa-solid fa-…">` with a fixed, typed icon map
  (`fa-circle-check`, `fa-circle-xmark`, `fa-circle-info`, `fa-mobile-screen`, `fa-display`, `fa-rotate`,
  `fa-play`, `fa-stop`, `fa-screwdriver-wrench`, `fa-arrow-up-right-from-square`, `fa-folder-open`,
  `fa-triangle-exclamation`).
- Icons are **grayscale only**: the font inherits `currentColor`, and no rule in `app.css` gives an icon a
  hue. State is carried by glyph *shape* plus text, never colour.
- The previous `src/lib/Icon.svelte` + `src/lib/icons.ts` (definitions) are **deleted**, so there is
  exactly one icon system in the tree.

### Licence obligations (hard requirement)

Font Awesome Free is triple-licensed: **icons CC BY 4.0, fonts SIL OFL 1.1, code MIT**. All three require
attribution in a distributed build. `NOTICE.md` carries the attribution block and a credits line appears in
the setup screen footer. `LICENSE` is the upstream MIT text and must not be edited.

## Consequences

| Metric | ADR-0008 (definitions) | ADR-0009 (font, solid only) |
|---|---|---|
| JS gzip | 17.14 kB | smaller (no path data) |
| Font asset | none | **one** `fa-solid-900.woff2` (~150 kB) |
| Icons covered | 4 | 13, and any future `fa-*` class is free |
| Runtime network | none | none |
| CSP | `font-src` unused | `font-src 'self'` required |
| Failure mode | none (inline SVG) | if the font fails to load, icons vanish — mitigated by same-origin bundling and the T7 offline test |

The trade is ~150 kB of installer for a maintainable icon system with no per-glyph bookkeeping. The bundle
budget (`14-frontend-design.md` §6: 180 kB gzip JS, 25 kB gzip CSS) is unaffected; the font is a separate
asset class and is recorded in `21-mvp-log.md` §6.

**Offline proof is a required test**: with the network disabled the app must render icons (T7 step 4d). If
icons disappear offline, the import or CSP is wrong and the milestone is not done.