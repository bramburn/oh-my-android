<script lang="ts">
  /**
   * Thin wrapper over the Font Awesome CSS classes.
   *
   * The font is **bundled offline** (`@fortawesome/fontawesome-free`, woff2 emitted by Vite into `dist/`),
   * never a CDN — the app must work with the network disabled (ADR-0009). Icons are decorative: every one
   * of them sits next to a text label, so they are `aria-hidden`.
   */
  export type IconName =
    | 'circle-check'
    | 'circle-xmark'
    | 'circle-info'
    | 'mobile-screen'
    | 'display'
    | 'rotate'
    | 'play'
    | 'stop'
    | 'screwdriver-wrench'
    | 'arrow-up-right-from-square'
    | 'folder-open'
    | 'triangle-exclamation';

  interface Props {
    name: IconName;
    /** Extra classes, e.g. `lg` for a larger glyph. */
    class?: string;
  }

  let { name, class: className = '' }: Props = $props();
</script>

<!-- `class` is a Svelte attribute; the icon name is a compile-time union so no user data reaches it. -->
<i class="fa-solid fa-{name} {className}" aria-hidden="true"></i>