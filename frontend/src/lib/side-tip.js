import { subscribeTheme } from './theme.js';

const SHOW_DELAY = 400;
const HIDE_DELAY = 120;

const tip = document.createElement('div');
tip.className = 'sidebar-tooltip';

const text = document.createElement('div');
text.className = 'sidebar-tooltip-text';

const arrow = document.createElement('div');
arrow.className = 'sidebar-tooltip-arrow';

tip.append(text, arrow);
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

function reveal(node) {
  const label = node.getAttribute('data-tip');
  if (!label) {
    hideNow();
    return;
  }
  text.textContent = label;
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
    if (tip.classList.contains('is-visible')) text.textContent = label;
  });
  watcher.observe(node, { attributes: true, attributeFilter: ['data-tip'] });
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

  current = node;
  watch(node);
  if (tip.classList.contains('is-visible')) reveal(node);
  else showTimer = setTimeout(() => reveal(node), SHOW_DELAY);
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
