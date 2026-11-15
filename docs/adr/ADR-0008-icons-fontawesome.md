# ADR-0008 — Font Awesome Free for UI icons

- **Status:** **Superseded by ADR-0009** (2026-11-15) — kept for the record; the icon-definitions approach
  was replaced by the bundled icon font when the icon count grew from 4 to 13
- **Supersedes:** nothing
- **Related:** `docs/windows-port/14-frontend-design.md` (monochrome design), `src/AGENTS.md` §9 (no new
  runtime dependency without an ADR)

## Context

The MVP shipped with text-only buttons and no iconography. Icons were requested for the three toolbar
actions and the device row. The hard constraint is that the UI is **monochrome** — state is carried by text,
weight and border, never hue — so any icon set must tint from `currentColor` and never ship a second colour.

Candidates considered:

| Option | Cost | Verdict |
|---|---|---|
| **Font Awesome Free (solid), icon definitions only** | +4.5 KB raw / **+2.1 KB gzip** for 4 glyphs | **chosen** |
| Font Awesome Free as a **web font** (`@fortawesome/fontawesome-free` CSS + woff2) | ~400 KB of fonts for 4 glyphs; icons become text, can fail to load | rejected |
| `@fortawesome/svelte-fontawesome` (official Svelte wrapper) | wrapper still exports Svelte 4 `SvelteComponentTyped` types and imports `csstype`, which it does **not** declare as a peer → `svelte-check` fails | rejected |
| `@fortawesome/fontawesome-svg-core` `icon()` | works correctly, but pulls the core runtime (transforms, masks, CSS injection, DOM watcher): **+92 KB raw / +24 KB gzip** for four static glyphs | rejected |
| Hand-drawn inline SVG | zero deps, but a bespoke icon set is worse quality and unmaintainable | rejected |

## Decision

Use **`@fortawesome/free-solid-svg-icons`** as the icon source, and render it with a local 20-line component
(`src/lib/Icon.svelte`) that reads the documented `IconDefinition` shape directly:

```
icon = [width, height, aliases, unicode, pathData]   // pathData: string | string[]
```

- Pinned exactly: `@fortawesome/free-solid-svg-icons@7.3.1`.
- The package declares `sideEffects: false`, so importing four named icons lets Vite drop the ~2000 others.
- The SVG is emitted with `fill="currentColor"`, `aria-hidden="true"` and `role="presentation"`: icons are
  **decorative** and always sit next to a text label, so the monochrome rule holds by construction and no
  icon needs an accessible name of its own.
- No icon font, no CSS classes (`.fa-*`), no core runtime, no `{@html}`: the path data is rendered by Svelte
  itself, so the markup is inert and cannot introduce XSS surface.
- Sizing lives in `src/app.css` (`.icon svg { width: 1em; height: 1em }`), never inline style attributes —
  which also keeps the CSP free of `style-src 'unsafe-inline'` for the icon path.

### Licence obligation

Font Awesome Free icons are **CC BY 4.0** (the code is MIT; the icons are not). Attribution is required for
any distributed build, so `NOTICE.md` carries the attribution line and the licence URLs, and it must not be
removed. This is a hard requirement, not documentation hygiene: shipping the installer without it violates
the icon licence.

### Adding an icon

1. Add the named import to `src/lib/icons.ts` (one file, so the tree-shaking surface stays auditable).
2. Check the glyph exists: `node -e "const s=require('@fortawesome/free-solid-svg-icons');console.log(!!s.faX)"`.
3. Use `<Icon icon={faX} />` next to a text label. Never icon-only without an `aria-label` on the control.
4. Re-run `npm run build` and check the gzip delta (≈0.5 KB per glyph; the 180 KB budget is in
   `14-frontend-design.md` §6).

## Consequences

- Bundle: **38.4 kB → 42.9 kB raw (15.1 → 17.1 kB gzip)**, i.e. 9.5 % of the 180 KB budget.
- One more runtime dependency, pinned, MIT + CC BY 4.0, attribution recorded.
- Icons are inline SVG, so they render with the first paint (no font-loading flash, no FOUT, no layout shift).
- If the icon count ever exceeds ~40, revisit a sprite sheet; the per-glyph cost stays flat because each
  glyph is only its path data.