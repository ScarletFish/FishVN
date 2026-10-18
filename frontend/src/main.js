import { mount } from 'svelte';
import './css/styles.css';
import App from './App.svelte';
import { useScrollbars } from './lib/scrollbars.js';

const splash = document.getElementById('splash');

function hideSplash() {
  if (!splash || splash.classList.contains('is-hidden')) return;
  splash.classList.add('is-hidden');
  setTimeout(() => splash.remove(), 320);
}

if ('__TAURI_INTERNALS__' in window) {
  window.addEventListener('app:ready', hideSplash, { once: true });
  setTimeout(hideSplash, 8000);
} else {
  splash?.remove();
}

const app = mount(App, {
  target: document.getElementById('app'),
});

useScrollbars(document.querySelector('.shell__main'));

export default app;
