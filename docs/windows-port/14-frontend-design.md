# 14 — Frontend design (Svelte 5 + TS + Vite + Tailwind v4)

## 1. Screens and component tree

Mirrors upstream's window inventory (`docs/architecture/08-ui-ux-map.md`) minus MCP/agent UI.

```
src/
  main.ts
  App.svelte                      # router-ish switch on window label
  windows/
    Panel.svelte                  # 236–360 px wide, always-on-top, no-activate
      DeviceHeader.svelte         # 8-state switch, serial, model, Android ver
      FeatureGrid.svelte          # sections → FeatureTile[]
      ToastStack.svelte
    Settings.svelte               # SDK path, captures dir, start-at-login, updates, access policy
    Logcat.svelte                 # filter, pause, search, export
    FileBrowser.svelte            # device tree + local target, transfer queue
    Inspector.svelte              # (M5) dp measurements, design overlay
  components/
    FeatureTile.svelte            # one capability: label, state, on/off, busy, error
    Stepper.svelte                # font size / display size (7 steps each)
    Segmented.svelte              # dark mode, network profile
    ConfirmDialog.svelte          # destructive actions
    ProgressRow.svelte            # install / transfer
    EmptyState.svelte  ErrorState.svelte  Skeleton.svelte
  state/
    devices.svelte.ts             # $state rune store fed by events
    settings.svelte.ts
    actions.svelte.ts             # busy/error per feature id
    logcat.svelte.ts              # ring buffer, cap 50k lines
  lib/
    ipc.ts                        # typed wrappers over invoke/listen/Channel
    bindings.ts                   # GENERATED from Rust (see 12-ipc-contract.md §6)
    errors.ts                     # ErrorCode → user string + hint
    format.ts                     # dp/px, bytes, durations
```

## 2. State (runes, not stores)

Use Svelte 5 `$state`/`$derived` in `.svelte.ts` modules. One module per Rust-owned domain; the module is
the only place that subscribes to its event.

```ts
// state/devices.svelte.ts
let devices = $state<Device[]>([]);
let active = $state<string | null>(null);
let state = $derived(panelState(devices, active));   // the 8-state machine, pure fn

export function subscribe() {
  const un = listen<DevicesChanged>('devices://changed', e => {
    devices = e.payload.devices; active = e.payload.active;
  });
  invoke('devices_list').then(seed);
  return un;
}
```

Rules:

1. **No component calls `invoke` directly** — only `lib/ipc.ts` wrappers.
2. **Rust is the source of truth.** The UI never optimistically flips a toggle; it shows `pending` until
   `action://state` confirms (matches upstream's `FeatureStore` busy semantics).
3. Never hold device output in a component; logcat and file lists live in the store, components virtualize.

## 3. Lists and performance

| List | Size | Approach |
|---|---|---|
| Feature grid | ~40 tiles | plain DOM, no virtualization |
| Device list | <10 | plain |
| Logcat | up to 50 k lines | **virtualized**: hand-rolled fixed-row windowing (row = 20 px, overscan 20) — one dependency avoided; `@tanstack/svelte-virtual` if hand-rolling stalls |
| File browser | ≤5 k entries | virtualized grid, same primitive |
| APK list | <200 | plain |

Budgets: first paint ≤300 ms after window show; a 20 k-line/s logcat stream must keep the UI responsive at
60 fps (`17-testing-strategy.md` §6).

## 4. Windows feel

| Aspect | Decision |
|---|---|
| Title bar | **native decorations** for Settings/Logcat/FileBrowser; `decorations: false` + a drag region only for the panel (it is a floating tool window, upstream does the same with `NSPanel`) |
| Panel chrome | compact, 8 px radius, subtle drop shadow, `WS_EX_NOACTIVATE` so clicks do not steal focus from the emulator (`Sources/Windows/FloatingPanel.swift:22-40`) |
| Topmost | `alwaysOnTop: true` for the panel only, toggleable in settings |
| Theme | light/dark following the OS (`prefers-color-scheme`); Tailwind v4 `@theme` tokens; no third-party theme system |
| Mica/acrylic | **not** in v1; it interacts badly with `transparent: true` + always-on-top. Revisit in M5 |
| DPI | 100–200 % scaling; panel width in logical px; `devicePixelRatio` respected in the mirror canvas |
| Multi-monitor | remember position per monitor config via `window-state` plugin; if the saved monitor is gone, centre on the primary |
| Fonts | Segoe UI Variable → `system-ui` fallback; monospace for logcat (`Cascadia Mono`, `Consolas`) |
| Shortcuts | `Ctrl+Shift+A` toggle panel, `Ctrl+,` settings, `Ctrl+Shift+L` logcat, `Esc` close panel, `F5` refresh device, `Ctrl+K` command palette (M5) |
| Accessibility | full keyboard nav, visible focus rings, `aria-live` for toasts, no colour-only state (icons + text), 4.5:1 contrast |
| i18n | `svelte-i18n`-free scaffold: `messages/en.json` + `t()` helper; no second locale in v1 |

## 5. Loading / empty / error states

Every screen must render five states — this is where upstream is unusually strong (8 panel states) and
where a rewrite usually regresses:

1. **loading** — skeleton, never a spinner-only panel;
2. **empty** — explanation + primary action ("No device. Plug in a phone or start an emulator.");
3. **error** — `ErrorCode` → sentence + `hint` + retry button;
4. **busy** — per-tile progress, other tiles stay usable (only mutating actions serialize);
5. **degraded** — e.g. mirroring unavailable: keep every other feature working, banner explains why.

`EmptyState` and `ErrorState` are shared components so no screen can invent its own wording.

## 6. Asset budget

| Asset | Budget |
|---|---|
| JS (gzip) | ≤180 KB initial route |
| CSS (gzip) | ≤25 KB |
| Fonts | none bundled in v1 (system fonts) |
| Icons | inline SVG sprite ≤15 KB |
| Total frontend in installer | ≤2 MB |

No icon font, no component library (Skeleton/Flowbite/etc. would add 40–120 KB for what six components do),
no animation library.

## 7. Build

```
npm run dev        # vite (HMR) with the Tauri dev window
npm run build      # vite build → dist/
npm run check      # svelte-check + tsc --noEmit
npm run test       # vitest (unit + component)
npm run lint       # eslint + prettier --check
npm run check:bindings   # regenerate bindings.ts, diff, fail on drift
```

All of the above are `UNVERIFIED` — nothing is scaffolded yet (`18-plan-and-backlog.md` M0).