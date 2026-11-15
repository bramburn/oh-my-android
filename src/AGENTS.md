# AGENTS.md — src (scope: src/, the frontend)

Svelte 5 + TypeScript + Vite. Read the root `AGENTS.md` first.

## Rules

1. **Monochrome.** No hue anywhere — no colour tokens, no coloured icons, no gradient. State is carried by
   text, weight and border (`src/app.css`). Grayscale is the design, not a theme.
2. **`src/lib/ipc.ts` is the only module that calls `invoke`.** Components import typed wrappers. A component
   that imports `@tauri-apps/api` directly is a review reject.
3. **No `any`.** Strict TS with `noUncheckedIndexedAccess` and `exactOptionalPropertyTypes`.
   `npm run check` (svelte-check) must be clean.
4. **One shared busy flag.** While any command is in flight, all action buttons are disabled — upstream drops
   concurrent taps (`Sources/State/FeatureStore.swift:143`) and we do the same.
5. **Never show stale data.** On a failed refresh the list is cleared before the error state renders
   (override O1 in `docs/windows-port/21-mvp-log.md` §2).
6. **Every state is explicit and rendered**: `loading`, `sdkMissing`, `error`, `ready`, and the empty case
   inside `ready`. No screen may fall through to a blank panel.
7. **Destructive actions confirm.** `Stop adb` calls `confirm(...)`; that dialog is the human approval for
   `kill-server` (override O2). Do not remove it, and do not add a second path to `adb_stop`.
8. **Accessibility floor**: keyboard reachable, `:focus-visible` outline, min 40 px hit targets, `role="alert"`
   on errors.
9. **No new runtime dependency** (no component library, no state library, no icon font) without an ADR —
   the bundle budget is 180 KB gzip JS (`docs/windows-port/14-frontend-design.md` §6); the MVP is at 17 KB.
   The one approved exception is Font Awesome **icon definitions** (`docs/adr/ADR-0008-icons-fontawesome.md`).
10. **Icons come from the bundled Font Awesome font** (`src/lib/FaIcon.svelte`). No other icon set, no
    `.fa-*` CSS classes, no `@fortawesome/fontawesome-svg-core`, no `@fortawesome/svelte-fontawesome`
    (both rejected in ADR-0008 with measured numbers). Import named glyphs only, so tree-shaking holds
    (~0.5 KB gzip per glyph). Icons are decorative: always pair with a text label, never icon-only.
11. **Font Awesome icons are CC BY 4.0.** The attribution block in `NOTICE.md` is a licence condition of the
    shipped installer, not documentation. Do not delete or move it.

## Files

| File | Owns |
|---|---|
| `src/main.ts` | mount point |
| `src/App.svelte` | the single screen: states, list, toolbar |
| `src/app.css` | the monochrome design tokens + layout |
| `src/lib/ipc.ts` | typed command wrappers, `Device`/`AppError` types, error helpers |
| `src/lib/FaIcon.svelte` | the whole icon surface: a typed glyph union rendered as `<i class="fa-solid fa-…">` (ADR-0009) |
| `src/lib/SetupScreen.svelte` | the setup guide — shown **only** when no runnable adb was found |
| `src/lib/DevicePanel.svelte` | the device list, Refresh/Start/Stop, error + empty states |
| `vite.config.ts` | fixed port 1420, `clearScreen: false` (Tauri logs share the terminal) |

## Adding a UI feature

1. Add the wrapper to `ipc.ts` (typed, no `any`) and the Rust command with the same name.
2. Add the state to `App.svelte`'s `Phase` union if it is a new *condition*, not a new *value*.
3. Keep the busy flag honest: `busy = true` in `try`, `busy = false` in `finally`.
4. Add the copy for the failure case — a code without a sentence is incomplete work.
5. Check `npm run build` output size before and after (and eyeball a screenshot — a `:global()` selector in
   `app.css` silently kills icon sizing, because `app.css` is a plain global stylesheet, not a component).