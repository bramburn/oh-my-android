/**
 * The icon set, and the only place Font Awesome is imported.
 *
 * Font Awesome Free icons are CC BY 4.0 — attribution lives in `NOTICE.md` and `LICENSE`.
 * Importing named icons (not the whole library) is what keeps this tree-shakeable:
 * `free-solid-svg-icons` declares `sideEffects: false`, so Vite drops the ~2000 unused glyphs.
 */
import {
  faArrowsRotate,
  faMobileScreen,
  faPlay,
  faStop,
} from '@fortawesome/free-solid-svg-icons';

export { faArrowsRotate, faMobileScreen, faPlay, faStop };