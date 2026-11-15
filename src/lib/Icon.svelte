<script lang="ts">
  /**
   * Renders a Font Awesome icon as inline SVG, straight from its icon definition.
   *
   * Why not the obvious options:
   * - **Web font** (`@fortawesome/fontawesome-free` CSS): ~400 KB of woff2 for four glyphs.
   * - **`@fortawesome/svelte-fontawesome`** (v0.2.4): still ships Svelte 4 legacy typings and imports
   *   `csstype`, which is an undeclared peer — `svelte-check` fails on it.
   * - **`@fortawesome/fontawesome-svg-core`** `icon()`: works, but costs +92 KB raw / +24 KB gzip because
   *   the core runtime (transforms, masks, CSS injection, DOM watcher) ships for four static glyphs.
   *
   * An {@html}-free render needs only `icon = [width, height, aliases, unicode, pathData]`, which is a
   * stable, documented shape of `IconDefinition`. `currentColor` keeps it monochrome by construction, and
   * the definition is build-time trusted content, never user input. Decorative by design (`aria-hidden`):
   * every icon in this app sits next to a text label.
   */
  import type { IconDefinition } from '@fortawesome/free-solid-svg-icons';

  interface Props {
    icon: IconDefinition;
    class?: string;
  }

  let { icon, class: className = '' }: Props = $props();

  const size = $derived({ width: icon.icon[0], height: icon.icon[1] });
  const paths = $derived(
    typeof icon.icon[4] === 'string' ? [icon.icon[4]] : icon.icon[4],
  );
</script>

<span class="icon {className}" aria-hidden="true">
  <svg
    viewBox="0 0 {size.width} {size.height}"
    role="presentation"
    focusable="false"
    fill="currentColor"
  >
    {#each paths as path}
      <path d={path} />
    {/each}
  </svg>
</span>