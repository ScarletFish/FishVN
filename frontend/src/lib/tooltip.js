
import { createTipBody } from './tip-body.js';
import { onTipInterrupt, tipsSuppressed } from './tip-guard.js';

const SHOW_DELAY = 500;
const HIDE_DELAY = 80;

const el = document.createElement('div');
el.id = 'globalTooltip';

const body = createTipBody();
el.append(body.root);

document.body.appendChild(el);

let showTimer = null;
let hideTimer = null;
let current = null;

function hideNow() {
  clearTimeout(showTimer);
  clearTimeout(hideTimer);
  current = null;
  el.classList.add('is-exiting');
  el.classList.remove('is-visible');
}

onTipInterrupt(hideNow);

document.addEventListener('mouseover', (e) => {
  const target = e.target.closest('[data-tooltip]');
  if (target === current) return;

  clearTimeout(showTimer);
  clearTimeout(hideTimer);
  if (current && !current.isConnected) {
    current = null;
    el.classList.add('is-exiting');
    el.classList.remove('is-visible');
  }
  if (!target) return;
  if (tipsSuppressed()) return;

  current = target;
  const entryX = e.clientX;
  const entryY = e.clientY;

  showTimer = setTimeout(() => {
    if (tipsSuppressed()) return;
    el.classList.toggle('is-rich', target.hasAttribute('data-tooltip-rich'));
    body.set(target.getAttribute('data-tooltip'), target.getAttribute('data-tooltip-key'));
    position(entryX, entryY);
    el.classList.remove('is-exiting');
    el.classList.add('is-visible');
  }, SHOW_DELAY);
});

document.addEventListener('mouseout', (e) => {
  const target = e.target.closest('[data-tooltip]');
  if (!target) return;
  if (target !== current && !target.contains(current)) return;

  clearTimeout(showTimer);
  current = null;
  hideTimer = setTimeout(() => {
    el.classList.add('is-exiting');
    el.classList.remove('is-visible');
  }, HIDE_DELAY);
});

document.addEventListener(
  'scroll',
  () => {
    if (!el.classList.contains('is-visible')) return;
    hideNow();
  },
  true,
);

function position(cx, cy) {
  const tw = el.offsetWidth || 0;
  const th = el.offsetHeight || 30;
  const gap = 10;

  let left = cx + gap;
  let top = cy + gap + 4;

  if (tw && left + tw > window.innerWidth - 6) left = cx - tw - gap;
  if (top + th > window.innerHeight - 6) top = cy - th - gap;

  el.style.left = Math.max(6, left) + 'px';
  el.style.top = Math.max(6, top) + 'px';
}