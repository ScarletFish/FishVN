const hooks = new Set();
let pressed = false;
let holds = 0;

export function onTipInterrupt(fn) {
  hooks.add(fn);
  return () => hooks.delete(fn);
}

export function interruptTips() {
  for (const fn of hooks) fn();
}

export function holdTips() {
  holds += 1;
  interruptTips();
  return () => {
    holds = Math.max(0, holds - 1);
  };
}

export function tipsSuppressed() {
  return pressed || holds > 0;
}

document.addEventListener(
  'pointerdown',
  (event) => {
    if (event.button === 0) pressed = true;
    interruptTips();
  },
  true,
);

document.addEventListener('pointerup', () => {
  pressed = false;
});

document.addEventListener('pointercancel', () => {
  pressed = false;
});

document.addEventListener('selectionchange', () => {
  const sel = window.getSelection();
  if (sel && !sel.isCollapsed && sel.toString().trim()) interruptTips();
});
