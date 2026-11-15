import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Tauri expects a fixed dev port and no screen clearing (its own logs share the terminal).
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ['**/src-tauri/**', '**/crates/**', '**/target/**'] },
  },
  build: {
    target: 'chrome105',
    sourcemap: false,
    emptyOutDir: true,
  },
});