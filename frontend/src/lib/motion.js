const REDUCE_MOTION_KEY = 'fishvn.reduceMotion';

let systemReduce = null;

function reduceMedia() {
  if (!systemReduce) systemReduce = window.matchMedia('(prefers-reduced-motion: reduce)');
  return systemReduce;
}

function applyToDocument(on) {
  if (on) document.documentElement.dataset.reduceMotion = 'on';
  else delete document.documentElement.dataset.reduceMotion;
}

export function getReduceMotion() {
  if (localStorage.getItem(REDUCE_MOTION_KEY) === '1') return true;
  return reduceMedia().matches;
}

export function setReduceMotion(on) {
  localStorage.setItem(REDUCE_MOTION_KEY, on ? '1' : '0');
  applyToDocument(getReduceMotion());
  return on;
}

export function shouldReduceMotion() {
  return getReduceMotion();
}

export function syncReduceMotion() {
  applyToDocument(getReduceMotion());
  reduceMedia().addEventListener('change', () => applyToDocument(getReduceMotion()));
}
