import './app.css';
// Font Awesome, bundled offline: Vite copies `fa-solid-900.woff2` into `dist/assets/` at build time.
// `fontawesome.min.css` + `solid.min.css` (NOT `all.min.css`) so the Regular and Brands faces — never used
// here — are not shipped. No CDN, no runtime network; CSP is `font-src 'self'` (ADR-0009).
import '@fortawesome/fontawesome-free/css/fontawesome.min.css';
import '@fortawesome/fontawesome-free/css/solid.min.css';
import { mount } from 'svelte';
import App from './App.svelte';

const target = document.getElementById('app');
if (target === null) {
  throw new Error('index.html is missing #app');
}

const app = mount(App, { target });

export default app;