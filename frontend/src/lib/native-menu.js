const EDITABLE = 'input, textarea, [contenteditable]:not([contenteditable="false"])';

export function keepsNativeMenu(target) {
  return target instanceof Element && Boolean(target.closest(EDITABLE));
}

document.addEventListener('contextmenu', (event) => {
  if (event.defaultPrevented) return;
  if (keepsNativeMenu(event.target)) return;
  event.preventDefault();
});
