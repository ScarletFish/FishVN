import { subscribeTheme } from './theme.js';
import { createTipBody } from './tip-body.js';
import { onTipInterrupt, tipsSuppressed } from './tip-guard.js';

const SHOW_DELAY = 400;
const HIDE_DELAY = 120;

const tip = document.createElement('div');
tip.className = 'sidebar-tooltip';

const bubble = document.createElement('div');
bubble.className = 'sidebar-tooltip-text';

const body = createTipBody();
bubble.append(body.root);

const arrow = document.createElement('div');
arrow.className = 'sidebar-tooltip-arrow';

tip.append(bubble, arrow);
document.body.appendChild(tip);

let showTimer = null;
let hideTimer = null;
let current = null;
let watcher = null;

function clearTimers() {
  clearTimeout(showTimer);
  clearTimeout(hideTimer);
  showTimer = null;
  hideTimer = null;
}

function unwatch() {
  if (!watcher) return;
  watcher.disconnect();
  watcher = null;
}

function hideNow() {
  clearTimers();
  unwatch();
  current = null;
  tip.classList.remove('is-visible');
}

function hideInstant() {
  if (!tip.classList.contains('is-visible')) return;
  tip.style.transition = 'none';
  tip.classList.remove('is-visible');
  requestAnimationFrame(() => {
    tip.style.transition = '';
  });
}

onTipInterrupt(hideNow);

function apply(node, label) {
  body.set(label, node.getAttribute('data-tip-key'));
}

function reveal(node) {
  const label = node.getAttribute('data-tip');
  if (!label) {
    hideNow();
    return;
  }
  apply(node, label);
  const box = node.getBoundingClientRect();
  tip.style.top = box.top + box.height / 2 + 'px';
  tip.classList.add('is-visible');
}

function watch(node) {
  unwatch();
  watcher = new MutationObserver(() => {
    const label = node.getAttribute('data-tip');
    if (!label) {
      hideNow();
      return;
    }
    if (tip.classList.contains('is-visible')) apply(node, label);
  });
  watcher.observe(node, { attributes: true, attributeFilter: ['data-tip', 'data-tip-key'] });
}

document.addEventListener('mouseover', (event) => {
  const node = event.target.closest('[data-tip]');
  if (node === current) return;

  clearTimers();
  if (current && !current.isConnected) {
    current = null;
    unwatch();
    tip.classList.remove('is-visible');
  }
  if (!node || !node.getAttribute('data-tip')) return;
  if (tipsSuppressed()) return;

  current = node;
  watch(node);
  if (tip.classList.contains('is-visible')) reveal(node);
  else
    showTimer = setTimeout(() => {
      if (tipsSuppressed()) return;
      reveal(node);
    }, SHOW_DELAY);
});

document.addEventListener('mouseout', (event) => {
  const node = event.target.closest('[data-tip]');
  if (!node || node !== current) return;
  if (event.relatedTarget && node.contains(event.relatedTarget)) return;

  clearTimers();
  unwatch();
  current = null;
  hideTimer = setTimeout(() => tip.classList.remove('is-visible'), HIDE_DELAY);
});

subscribeTheme(hideInstant);
