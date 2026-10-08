import { mount } from 'svelte';
import './css/styles.css';
import App from './App.svelte';
import { useScrollbars } from './lib/scrollbars.js';

const app = mount(App, {
  target: document.getElementById('app'),
});

useScrollbars(document.querySelector('.shell__main'));

export default app;
