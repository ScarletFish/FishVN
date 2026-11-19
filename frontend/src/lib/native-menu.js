import { openEditMenuAt } from './edit-menu.svelte.js';

document.addEventListener('contextmenu', (event) => {
  if (event.defaultPrevented) return;
  event.preventDefault();
  openEditMenuAt(event);
});
